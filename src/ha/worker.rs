use std::path::Path;

use crate::{
    dcs::{ClusterMemberView, DcsQuorumState, DcsView, MemberPostgresView},
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    postgres_roles,
    process::jobs::{ActiveJobKind, PostgresStartIntent, ProcessIntent},
    state::{LeaseEpoch, MemberId, PgEndpoint, WorkerError, WorkerStatus},
};

use super::{
    decide::decide,
    process_dispatch::{dispatch_process_action, ProcessDispatchError},
    reconcile::reconcile,
    state::{HaRuntimeCtx, HaState},
    types::{
        last_start_success_at, last_success_at, wal_position, ApiVisibility, CoordinationAction,
        CoordinationState, DataDirState, DesiredState, DivergenceState, ElectionEligibility,
        GlobalKnowledge, IneligibleReason, LeadershipView, LocalAction, LocalDataState,
        LocalKnowledge, ObservationState, ObservedPrimary, PeerKnowledge, PeerLeaderState,
        PostgresState, PrimaryObservation, ProcessAssessment, PublicationGoal, PublicationState,
        QuorumCoordinationState, ReconcilePlan, ReplicationState, StorageState, WalPosition,
        WorldView,
    },
};

pub(crate) async fn run(mut ctx: HaRuntimeCtx) -> Result<(), WorkerError> {
    let mut interval = tokio::time::interval(ctx.cadence.poll_interval);
    loop {
        tokio::select! {
            changed = ctx.observed.pg.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha pg subscriber closed: {err}")))?;
            }
            changed = ctx.observed.dcs.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha dcs subscriber closed: {err}")))?;
            }
            changed = ctx.observed.process.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha process subscriber closed: {err}")))?;
            }
            changed = ctx.observed.config.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha config subscriber closed: {err}")))?;
            }
            _ = interval.tick() => {}
        }
        step_once(&mut ctx).await?;
    }
}

pub(crate) async fn step_once(ctx: &mut HaRuntimeCtx) -> Result<(), WorkerError> {
    let now = (ctx.cadence.now)()?;
    let world = observe(ctx, now)?;
    let desired = decide(&world, &ctx.identity.member_id);
    let plan = reconcile(&world, &desired);
    let next_state = build_next_state(&ctx.state_channel.current, &world, &desired, &plan);

    ctx.state_channel
        .publisher
        .publish(next_state.clone())
        .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;
    ctx.state_channel.current = next_state;

    execute_plan(ctx, ctx.state_channel.current.tick, &plan).await?;

    Ok(())
}

fn observe(ctx: &HaRuntimeCtx, now: crate::state::UnixMillis) -> Result<WorldView, WorkerError> {
    let config = ctx.observed.config.latest();
    let pg = ctx.observed.pg.latest();
    let dcs = ctx.observed.dcs.latest();
    let process = ctx.observed.process.latest();
    let previous_observation = &ctx.state_channel.current.world.local.observation;
    let data_dir_path = config.postgres.paths.data_dir.clone();
    let observed_primary = dcs
        .quorum_state()
        .and_then(|quorum| observed_primary_member(quorum, &ctx.identity.member_id));
    let current_local_timeline = pg_timeline(&pg);
    let current_local_system_identifier = pg_system_identifier(&pg);
    let observation = ObservationState {
        pg_observed_at: pg.last_refresh_at().unwrap_or(now),
        last_start_success_at: last_start_success_at(&process),
        last_basebackup_success_at: last_success_at(&process, ActiveJobKind::BaseBackup),
        last_promote_success_at: last_success_at(&process, ActiveJobKind::Promote),
        last_demote_success_at: last_success_at(&process, ActiveJobKind::Demote),
        last_local_timeline: current_local_timeline.or(previous_observation.last_local_timeline),
        last_local_system_identifier: current_local_system_identifier
            .or(previous_observation.last_local_system_identifier),
    };
    let process_assessment = ProcessAssessment::from(&process);
    let (dcs_local_timeline, dcs_local_system_identifier) =
        local_member_identity_fallback(&dcs, &ctx.identity.member_id, &observation);
    let (retained_local_timeline, retained_local_system_identifier) =
        retained_local_identity_fallback(&observation);
    let local_data_timeline = current_local_timeline
        .or(dcs_local_timeline)
        .or(retained_local_timeline);
    let local_system_identifier = current_local_system_identifier
        .or(dcs_local_system_identifier)
        .or(retained_local_system_identifier);

    let local = LocalKnowledge {
        data_dir: build_data_dir_state(
            data_dir_path.as_path(),
            local_data_timeline,
            local_system_identifier,
            &process_assessment,
            &observed_primary,
        ),
        postgres: build_local_postgres_state(&pg, &dcs),
        process: process_assessment,
        storage: build_storage_state(
            &dcs,
            &pg,
            config.ha.lease_ttl_ms,
            &ctx.identity.member_id,
            now,
        ),
        managed_roles_reconciled: ctx.state_channel.current.managed_roles_reconciled,
        publication: ctx.state_channel.current.publication.clone(),
        observation,
    };
    let global = build_global_knowledge(&dcs, &pg, &local.data_dir, &ctx.identity.member_id);

    Ok(WorldView { local, global })
}

fn local_member_identity_fallback(
    dcs: &DcsView,
    self_id: &MemberId,
    observation: &ObservationState,
) -> (Option<u64>, Option<u64>) {
    if observation.basebackup_completed_awaiting_start() {
        return (None, None);
    }

    let member = dcs.member(self_id);
    (
        member.and_then(member_timeline),
        member.and_then(member_system_identifier),
    )
}

fn retained_local_identity_fallback(observation: &ObservationState) -> (Option<u64>, Option<u64>) {
    if observation.basebackup_completed_awaiting_start() {
        return (None, None);
    }

    (
        observation.last_local_timeline,
        observation.last_local_system_identifier,
    )
}

fn build_next_state(
    current: &HaState,
    world: &WorldView,
    desired: &DesiredState,
    plan: &ReconcilePlan,
) -> HaState {
    HaState {
        worker: WorkerStatus::Running,
        tick: current.tick.saturating_add(1),
        managed_roles_reconciled: next_managed_roles_reconciled(current, plan),
        publication: apply_publication_goal(&current.publication, &desired.publication),
        role: desired.role.clone(),
        world: world.clone(),
        clear_switchover: desired.clear_switchover,
        planned_actions: super::types::PlannedActions::from_plan(plan),
    }
}

fn next_managed_roles_reconciled(current: &HaState, plan: &ReconcilePlan) -> bool {
    if matches!(
        plan.process,
        Some(ProcessIntent::Bootstrap)
            | Some(ProcessIntent::ProvisionReplica(_))
            | Some(ProcessIntent::Start(PostgresStartIntent::DetachedStandby))
            | Some(ProcessIntent::Start(PostgresStartIntent::Replica { .. }))
    ) {
        return false;
    }

    current.managed_roles_reconciled
}

fn apply_publication_goal(current: &PublicationState, goal: &PublicationGoal) -> PublicationState {
    match goal {
        PublicationGoal::KeepCurrent => current.clone(),
        PublicationGoal::Publish(projection) => PublicationState::Projected(projection.clone()),
    }
}

async fn execute_plan(
    ctx: &mut HaRuntimeCtx,
    ha_tick: u64,
    plan: &ReconcilePlan,
) -> Result<(), WorkerError> {
    if let Some(action) = &plan.coordination {
        execute_coordination_action(ctx, ha_tick, 0, action).await?;
    }
    if let Some(action) = &plan.local {
        execute_local_action(ctx, ha_tick, 1, action).await?;
    }
    if let Some(action) = &plan.process {
        execute_process_action(ctx, ha_tick, 2, action).await?;
    }
    Ok(())
}

async fn execute_coordination_action(
    ctx: &mut HaRuntimeCtx,
    ha_tick: u64,
    action_index: usize,
    action: &CoordinationAction,
) -> Result<(), WorkerError> {
    match action {
        CoordinationAction::AcquireLease(_kind) => {
            ctx.control.dcs_handle.acquire_leadership().map_err(|err| {
                WorkerError::Message(format!(
                    "ha acquire lease failed at tick {ha_tick} index {action_index}: {err}"
                ))
            })
        }
        CoordinationAction::ReleaseLease => {
            ctx.control.dcs_handle.release_leadership().map_err(|err| {
                WorkerError::Message(format!(
                    "ha release lease failed at tick {ha_tick} index {action_index}: {err}"
                ))
            })
        }
        CoordinationAction::ClearSwitchover => {
            ctx.control.dcs_handle.clear_switchover().map_err(|err| {
                WorkerError::Message(format!(
                    "ha clear switchover failed at tick {ha_tick} index {action_index}: {err}"
                ))
            })
        }
    }
}

async fn execute_local_action(
    ctx: &mut HaRuntimeCtx,
    ha_tick: u64,
    action_index: usize,
    action: &LocalAction,
) -> Result<(), WorkerError> {
    match action {
        LocalAction::ReconcileManagedRoles => {
            let runtime_config = ctx.observed.config.latest();
            postgres_roles::reconcile_managed_roles(
                &runtime_config,
                runtime_config.postgres_socket_dir().as_path(),
                runtime_config.postgres.network.listen_port,
            )
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "ha reconcile managed roles failed at tick {ha_tick} index {action_index}: {err}"
                ))
            })?;
            ctx.state_channel.current.managed_roles_reconciled = true;
            Ok(())
        }
    }
}

async fn execute_process_action(
    ctx: &mut HaRuntimeCtx,
    ha_tick: u64,
    action_index: usize,
    action: &ProcessIntent,
) -> Result<(), WorkerError> {
    let runtime_config = ctx.observed.config.latest();
    dispatch_process_action(ctx, ha_tick, action_index, action, &runtime_config)
        .map(|_| ())
        .map_err(|err| map_process_dispatch_error(ha_tick, action_index, err))
}

fn map_process_dispatch_error(
    ha_tick: u64,
    action_index: usize,
    err: ProcessDispatchError,
) -> WorkerError {
    WorkerError::Message(format!(
        "ha process dispatch failed at tick {ha_tick} index {action_index}: {err}"
    ))
}

fn build_data_dir_state(
    data_dir: &Path,
    local_timeline: Option<u64>,
    local_system_identifier: Option<u64>,
    process: &ProcessAssessment,
    observed_primary: &Option<ObservedPrimary>,
) -> DataDirState {
    let pg_version_path = data_dir.join("PG_VERSION");
    if !data_dir.exists() {
        return DataDirState::Missing;
    }
    if !pg_version_path.exists() {
        return DataDirState::Initialized(LocalDataState::BootstrapEmpty);
    }

    let local_state = match observed_primary {
        Some(ObservedPrimary {
            system_identifier: Some(primary_system_identifier),
            ..
        }) if local_system_identifier.is_some()
            && local_system_identifier != Some(*primary_system_identifier) =>
        {
            LocalDataState::Diverged(DivergenceState::BasebackupRequired)
        }
        Some(ObservedPrimary {
            timeline: leader_timeline,
            ..
        }) if leader_timeline == &local_timeline => LocalDataState::ConsistentReplica,
        Some(ObservedPrimary {
            timeline: Some(_), ..
        }) if local_timeline.is_some() => LocalDataState::Diverged(DivergenceState::RewindPossible),
        Some(ObservedPrimary { .. })
            if matches!(
                process,
                ProcessAssessment::Failed(super::types::JobFailure {
                    job: ActiveJobKind::PgRewind,
                    recovery: super::types::FailureRecovery::FallbackToBasebackup,
                })
            ) =>
        {
            LocalDataState::Diverged(DivergenceState::BasebackupRequired)
        }
        _ => LocalDataState::ConsistentReplica,
    };

    DataDirState::Initialized(local_state)
}

fn build_postgres_state(pg: &PgInfoState) -> PostgresState {
    match pg {
        PgInfoState::Unknown { common } if common.sql != SqlStatus::Healthy => {
            PostgresState::Offline
        }
        PgInfoState::Unknown { .. } => PostgresState::Offline,
        PgInfoState::Primary {
            common, wal_lsn, ..
        } if common.sql == SqlStatus::Healthy => PostgresState::Primary {
            committed_lsn: wal_lsn.0,
        },
        PgInfoState::Primary { .. } => PostgresState::Offline,
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } if common.sql == SqlStatus::Healthy => PostgresState::Replica {
            upstream: upstream.as_ref().map(|value| value.member_id.clone()),
            replication: build_replication_state(common.timeline, *replay_lsn, *follow_lsn),
        },
        PgInfoState::Replica { .. } => PostgresState::Offline,
    }
}

fn build_local_postgres_state(pg: &PgInfoState, dcs: &DcsView) -> PostgresState {
    match build_postgres_state(pg) {
        PostgresState::Replica {
            upstream,
            replication,
        } => PostgresState::Replica {
            upstream: upstream.or_else(|| resolve_replica_upstream(pg, dcs)),
            replication,
        },
        state => state,
    }
}

fn build_replication_state(
    timeline: Option<crate::state::TimelineId>,
    replay_lsn: crate::state::WalLsn,
    follow_lsn: Option<crate::state::WalLsn>,
) -> ReplicationState {
    if let Some(position) = wal_position(timeline, follow_lsn) {
        return ReplicationState::Streaming(position);
    }
    if replay_lsn.0 > 0 {
        return ReplicationState::CatchingUp(WalPosition {
            timeline: timeline.map_or(0, |value| u64::from(value.0)),
            lsn: replay_lsn.0,
        });
    }
    ReplicationState::Stalled
}

fn build_storage_state(
    dcs: &DcsView,
    pg: &PgInfoState,
    lease_ttl_ms: u64,
    self_id: &MemberId,
    now: crate::state::UnixMillis,
) -> StorageState {
    let self_member = dcs.member(self_id);
    let pg_observation_stale = pg
        .last_refresh_at()
        .is_none_or(|last_refresh_at| now.0.saturating_sub(last_refresh_at.0) > lease_ttl_ms);
    if matches!(
        pg,
        PgInfoState::Primary { common, .. } if common.sql == SqlStatus::Healthy
    ) && (self_member.is_none() || pg_observation_stale)
    {
        StorageState::Stalled
    } else {
        StorageState::Healthy
    }
}

fn build_global_knowledge(
    dcs: &DcsView,
    pg: &PgInfoState,
    local_data_dir: &DataDirState,
    self_id: &MemberId,
) -> GlobalKnowledge {
    let coordination = dcs
        .quorum_state()
        .map(|quorum| {
            let leadership = build_leadership_view(quorum, self_id);
            let peers = quorum
                .members()
                .filter(|(member_id, _)| *member_id != self_id)
                .map(|(member_id, member)| {
                    (member_id.clone(), build_peer_knowledge_from_member(member))
                })
                .collect();
            let primary = observed_primary_member(quorum, self_id)
                .map(PrimaryObservation::Observed)
                .unwrap_or(PrimaryObservation::Absent);

            CoordinationState::Quorum(Box::new(QuorumCoordinationState {
                dcs: quorum.clone(),
                leadership,
                primary,
                switchover: quorum.switchover.clone(),
                peers,
            }))
        })
        .unwrap_or(CoordinationState::NoQuorum);

    GlobalKnowledge {
        coordination,
        self_peer: build_self_peer(pg, local_data_dir),
    }
}

fn build_peer_knowledge_from_member(member: &ClusterMemberView) -> PeerKnowledge {
    let api = ApiVisibility::Reachable;
    let readiness = member.postgres().readiness();
    let eligibility = match member.postgres() {
        MemberPostgresView::Unknown { .. } => {
            if readiness == Readiness::Ready {
                ElectionEligibility::BootstrapEligible
            } else {
                ElectionEligibility::Ineligible(IneligibleReason::NotReady)
            }
        }
        MemberPostgresView::Primary { .. } => {
            if readiness != Readiness::Ready {
                ElectionEligibility::Ineligible(IneligibleReason::NotReady)
            } else {
                member
                    .postgres()
                    .committed_wal()
                    .and_then(|value| wal_position(value.timeline, Some(value.lsn)))
                    .map(ElectionEligibility::PromoteEligible)
                    .unwrap_or(ElectionEligibility::Ineligible(IneligibleReason::Lagging))
            }
        }
        MemberPostgresView::Replica { .. } => {
            if readiness != Readiness::Ready {
                ElectionEligibility::Ineligible(IneligibleReason::NotReady)
            } else {
                member
                    .postgres()
                    .replay_wal()
                    .or_else(|| member.postgres().follow_wal())
                    .and_then(|value| wal_position(value.timeline, Some(value.lsn)))
                    .map(ElectionEligibility::PromoteEligible)
                    .unwrap_or(ElectionEligibility::Ineligible(IneligibleReason::Lagging))
            }
        }
    };

    PeerKnowledge { eligibility, api }
}

fn build_self_peer(pg: &PgInfoState, local_data_dir: &DataDirState) -> PeerKnowledge {
    let eligibility = match (local_data_dir, build_postgres_state(pg)) {
        (DataDirState::Missing, _)
        | (DataDirState::Initialized(LocalDataState::BootstrapEmpty), _) => {
            ElectionEligibility::BootstrapEligible
        }
        (_, PostgresState::Primary { committed_lsn }) => wal_position(
            pg_timeline_id(pg),
            Some(crate::state::WalLsn(committed_lsn)),
        )
        .map(ElectionEligibility::PromoteEligible)
        .unwrap_or(ElectionEligibility::BootstrapEligible),
        (_, PostgresState::Replica { .. }) => self_replica_position(pg)
            .map(ElectionEligibility::PromoteEligible)
            .unwrap_or(ElectionEligibility::Ineligible(IneligibleReason::Lagging)),
        (_, PostgresState::Offline) => {
            ElectionEligibility::Ineligible(IneligibleReason::StartingUp)
        }
    };
    PeerKnowledge {
        eligibility,
        api: ApiVisibility::Reachable,
    }
}

fn self_replica_position(pg: &PgInfoState) -> Option<WalPosition> {
    match pg {
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            ..
        } => wal_position(common.timeline, Some(*replay_lsn))
            .or_else(|| follow_lsn.and_then(|lsn| wal_position(common.timeline, Some(lsn)))),
        _ => None,
    }
}

fn resolve_replica_upstream(pg: &PgInfoState, dcs: &DcsView) -> Option<MemberId> {
    let primary_conninfo = match pg {
        PgInfoState::Replica { common, .. } => common.pg_config.primary_conninfo.as_ref(),
        _ => None,
    }?;
    let PgEndpoint::Tcp { host, port } = &primary_conninfo.endpoint else {
        return None;
    };

    dcs.members().find_map(|(member_id, member)| {
        (member.postgres_target().host() == host.as_str()
            && member.postgres_target().port() == *port)
            .then_some(member_id.clone())
    })
}

fn build_leadership_view(dcs: &DcsQuorumState, self_id: &MemberId) -> LeadershipView {
    let Some(epoch) = dcs.leadership.clone() else {
        return LeadershipView::Open;
    };
    if epoch.holder == *self_id {
        return LeadershipView::HeldBySelf(epoch);
    }

    match dcs.member(&epoch.holder) {
        None => LeadershipView::HeldByPeer {
            epoch,
            state: PeerLeaderState::Unreachable,
        },
        Some(member) => classify_foreign_leader(member, epoch),
    }
}

fn classify_foreign_leader(member: &ClusterMemberView, epoch: LeaseEpoch) -> LeadershipView {
    match member.postgres() {
        MemberPostgresView::Primary { .. } if member.postgres().readiness() == Readiness::Ready => {
            LeadershipView::HeldByPeer {
                epoch,
                state: PeerLeaderState::PrimaryReady,
            }
        }
        MemberPostgresView::Primary { .. } => LeadershipView::HeldByPeer {
            epoch,
            state: PeerLeaderState::Recovering,
        },
        MemberPostgresView::Unknown { .. } if member.postgres().readiness() == Readiness::Ready => {
            LeadershipView::HeldByPeer {
                epoch,
                state: PeerLeaderState::Unreachable,
            }
        }
        MemberPostgresView::Unknown { .. } | MemberPostgresView::Replica { .. } => {
            LeadershipView::HeldByPeer {
                epoch,
                state: PeerLeaderState::Recovering,
            }
        }
    }
}

fn observed_primary_member(dcs: &DcsQuorumState, self_id: &MemberId) -> Option<ObservedPrimary> {
    dcs.members().find_map(|(member_id, member)| {
        ((*member_id != *self_id)
            && matches!(member.postgres(), MemberPostgresView::Primary { .. })
            && member.postgres().readiness() == Readiness::Ready)
            .then(|| ObservedPrimary {
                member: member_id.clone(),
                timeline: member_timeline(member),
                system_identifier: member_system_identifier(member),
            })
    })
}

fn member_timeline(member: &ClusterMemberView) -> Option<u64> {
    member
        .postgres()
        .timeline()
        .map(|timeline| u64::from(timeline.0))
}

fn member_system_identifier(member: &ClusterMemberView) -> Option<u64> {
    member.postgres().system_identifier().map(|value| value.0)
}

fn pg_timeline(pg: &PgInfoState) -> Option<u64> {
    pg_timeline_id(pg).map(|timeline| u64::from(timeline.0))
}

fn pg_system_identifier(pg: &PgInfoState) -> Option<u64> {
    match pg {
        PgInfoState::Unknown { common }
        | PgInfoState::Primary { common, .. }
        | PgInfoState::Replica { common, .. } => common.system_identifier.map(|value| value.0),
    }
}

fn pg_timeline_id(pg: &PgInfoState) -> Option<crate::state::TimelineId> {
    match pg {
        PgInfoState::Unknown { common }
        | PgInfoState::Primary { common, .. }
        | PgInfoState::Replica { common, .. } => common.timeline,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        dcs::{ClusterMemberView, DcsView, MemberPostgresView},
        pginfo::conninfo::PgClientTls,
        pginfo::state::PgConnInfo,
    };
    use crate::{
        ha::types::{FailureRecovery, JobFailure},
        pginfo::state::{PgConfig, PgInfoCommon, Readiness, SqlStatus},
        state::{
            PgTcpTarget, SwitchoverState, SystemIdentifier, TimelineId, UnixMillis, WalLsn,
            WorkerStatus,
        },
    };

    use super::*;

    fn replica_pg_state(replay_lsn: u64, follow_lsn: Option<u64>) -> PgInfoState {
        PgInfoState::Replica {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: Some(TimelineId(7)),
                system_identifier: Some(SystemIdentifier(41)),
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: std::collections::BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(123)),
            },
            replay_lsn: WalLsn(replay_lsn),
            follow_lsn: follow_lsn.map(WalLsn),
            upstream: None,
        }
    }

    fn replica_pg_state_with_primary_conninfo(
        host: &str,
        port: u16,
    ) -> Result<PgInfoState, String> {
        let mut state = replica_pg_state(67_272_104, Some(67_272_104));
        if let PgInfoState::Replica { common, .. } = &mut state {
            common.pg_config.primary_conninfo = Some(PgConnInfo {
                endpoint: PgTcpTarget::new(host.to_string(), port)?,
                user: "replicator".to_string(),
                dbname: "postgres".to_string(),
                application_name: Some("node-a".to_string()),
                connect_timeout_s: Some(5),
                ssl_mode: crate::pginfo::state::PgSslMode::Require,
                ssl_root_cert: None,
                options: None,
                tls: PgClientTls {
                    mode: crate::pginfo::state::PgSslMode::Require,
                    root_cert: None,
                    client_cert: None,
                    client_key: None,
                },
            });
        }
        Ok(state)
    }

    fn dcs_view_for_member(member_id: &str, host: &str, port: u16) -> Result<DcsView, String> {
        Ok(DcsView::quorum(
            None,
            SwitchoverState::None,
            BTreeMap::from([(
                MemberId(member_id.to_string()),
                ClusterMemberView {
                    postgres_endpoint: PgTcpTarget::new(host.to_string(), port)?,
                    postgres: MemberPostgresView::Unknown {
                        common: PgInfoCommon {
                            worker: WorkerStatus::Running,
                            sql: SqlStatus::Healthy,
                            readiness: Readiness::Ready,
                            timeline: Some(TimelineId(7)),
                            system_identifier: Some(SystemIdentifier(41)),
                            pg_config: PgConfig {
                                port: None,
                                hot_standby: None,
                                primary_conninfo: None,
                                primary_slot_name: None,
                                extra: BTreeMap::new(),
                            },
                            last_refresh_at: Some(UnixMillis(123)),
                        },
                    },
                },
            )]),
        ))
    }

    #[test]
    fn self_peer_replica_eligibility_prefers_replay_lsn_over_follow_lsn() {
        let peer = build_self_peer(
            &replica_pg_state(67_272_104, Some(67_108_864)),
            &DataDirState::Initialized(LocalDataState::ConsistentReplica),
        );

        assert_eq!(
            peer.eligibility,
            ElectionEligibility::PromoteEligible(WalPosition {
                timeline: 7,
                lsn: 67_272_104,
            })
        );
    }

    #[test]
    fn local_postgres_state_resolves_replica_upstream_from_primary_conninfo() -> Result<(), String>
    {
        let state = build_local_postgres_state(
            &replica_pg_state_with_primary_conninfo("node-b", 5432)?,
            &dcs_view_for_member("node-b", "node-b", 5432)?,
        );

        assert_eq!(
            state,
            PostgresState::Replica {
                upstream: Some(MemberId("node-b".to_string())),
                replication: ReplicationState::Streaming(WalPosition {
                    timeline: 7,
                    lsn: 67_272_104,
                }),
            }
        );
        Ok(())
    }

    #[test]
    fn data_dir_state_requires_basebackup_for_mismatched_system_identifier() {
        let data_dir =
            std::env::temp_dir().join(format!("pgtm-ha-worker-test-{}", std::process::id()));
        let pg_version_path = data_dir.join("PG_VERSION");
        if data_dir.exists() {
            assert!(
                std::fs::remove_dir_all(&data_dir).is_ok(),
                "failed to clean test data dir"
            );
        }
        assert!(
            std::fs::create_dir_all(&data_dir).is_ok(),
            "failed to create test data dir"
        );
        assert!(
            std::fs::write(&pg_version_path, "16\n").is_ok(),
            "failed to create PG_VERSION"
        );
        let state = build_data_dir_state(
            &data_dir,
            Some(7),
            Some(41),
            &ProcessAssessment::Idle,
            &Some(ObservedPrimary {
                member: MemberId("node-c".to_string()),
                timeline: Some(8),
                system_identifier: Some(99),
            }),
        );
        assert!(
            std::fs::remove_dir_all(&data_dir).is_ok(),
            "failed to remove test data dir"
        );

        assert_eq!(
            state,
            DataDirState::Initialized(LocalDataState::Diverged(
                DivergenceState::BasebackupRequired
            ))
        );
    }

    #[test]
    fn basebackup_awaiting_restart_ignores_stale_dcs_local_identity() -> Result<(), String> {
        let data_dir =
            std::env::temp_dir().join(format!("pgtm-ha-worker-test-{}", std::process::id()));
        let pg_version_path = data_dir.join("PG_VERSION");
        if data_dir.exists() {
            std::fs::remove_dir_all(&data_dir)
                .map_err(|err| format!("failed to clean test data dir: {err}"))?;
        }
        std::fs::create_dir_all(&data_dir)
            .map_err(|err| format!("failed to create test data dir: {err}"))?;
        std::fs::write(&pg_version_path, "16\n")
            .map_err(|err| format!("failed to create PG_VERSION: {err}"))?;

        let observation = ObservationState {
            pg_observed_at: UnixMillis(100),
            last_start_success_at: Some(UnixMillis(10)),
            last_basebackup_success_at: Some(UnixMillis(20)),
            last_promote_success_at: None,
            last_demote_success_at: None,
            last_local_timeline: None,
            last_local_system_identifier: None,
        };
        let (local_timeline, local_system_identifier) = local_member_identity_fallback(
            &dcs_view_for_member("node-b", "node-b", 5432)?,
            &MemberId("node-b".to_string()),
            &observation,
        );
        let state = build_data_dir_state(
            &data_dir,
            local_timeline,
            local_system_identifier,
            &ProcessAssessment::Idle,
            &Some(ObservedPrimary {
                member: MemberId("node-a".to_string()),
                timeline: Some(8),
                system_identifier: Some(99),
            }),
        );

        std::fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("failed to remove test data dir: {err}"))?;

        if state != DataDirState::Initialized(LocalDataState::ConsistentReplica) {
            return Err(format!(
                "expected fresh basebackup to stop stale DCS identity from forcing another basebackup, got {state:?}"
            ));
        }

        Ok(())
    }

    #[test]
    fn retained_local_identity_keeps_last_seen_values_until_basebackup_restart_window() {
        let observation = ObservationState {
            pg_observed_at: UnixMillis(100),
            last_start_success_at: Some(UnixMillis(10)),
            last_basebackup_success_at: None,
            last_promote_success_at: None,
            last_demote_success_at: None,
            last_local_timeline: Some(1),
            last_local_system_identifier: Some(41),
        };

        assert_eq!(
            retained_local_identity_fallback(&observation),
            (Some(1), Some(41))
        );
    }

    #[test]
    fn retained_local_identity_ignores_stale_values_after_basebackup_until_pg_refresh() {
        let observation = ObservationState {
            pg_observed_at: UnixMillis(100),
            last_start_success_at: Some(UnixMillis(10)),
            last_basebackup_success_at: Some(UnixMillis(20)),
            last_promote_success_at: None,
            last_demote_success_at: None,
            last_local_timeline: Some(1),
            last_local_system_identifier: Some(41),
        };

        assert_eq!(retained_local_identity_fallback(&observation), (None, None));
    }

    #[test]
    fn rewind_failure_without_local_identity_requires_basebackup() -> Result<(), String> {
        let data_dir = std::env::temp_dir().join(format!(
            "pgtm-ha-worker-test-missing-identity-{}",
            std::process::id()
        ));
        let pg_version_path = data_dir.join("PG_VERSION");
        if data_dir.exists() {
            std::fs::remove_dir_all(&data_dir)
                .map_err(|err| format!("failed to clean test data dir: {err}"))?;
        }
        std::fs::create_dir_all(&data_dir)
            .map_err(|err| format!("failed to create test data dir: {err}"))?;
        std::fs::write(&pg_version_path, "16\n")
            .map_err(|err| format!("failed to create PG_VERSION: {err}"))?;

        let state = build_data_dir_state(
            &data_dir,
            None,
            None,
            &ProcessAssessment::Failed(JobFailure {
                job: ActiveJobKind::PgRewind,
                recovery: FailureRecovery::FallbackToBasebackup,
            }),
            &Some(ObservedPrimary {
                member: MemberId("node-a".to_string()),
                timeline: Some(8),
                system_identifier: Some(99),
            }),
        );

        std::fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("failed to remove test data dir: {err}"))?;

        if state
            != DataDirState::Initialized(LocalDataState::Diverged(
                DivergenceState::BasebackupRequired,
            ))
        {
            return Err(format!(
                "expected missing local identity to require basebackup, observed {state:?}"
            ));
        }

        Ok(())
    }
}
