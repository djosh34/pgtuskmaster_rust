use std::path::Path;

use crate::{
    dcs::{
        DcsLeaderStateView, DcsMemberPostgresView, DcsMemberView, DcsSwitchoverStateView,
        DcsSwitchoverTargetView, DcsView,
    },
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    postgres_roles,
    process::jobs::{ActiveJobKind, PostgresStartIntent, ProcessIntent},
    state::{MemberId, SystemIdentifier, TimelineId, WorkerError, WorkerStatus},
};

use super::{
    decide::decide,
    process_dispatch::{dispatch_process_action, ProcessDispatchError},
    reconcile::reconcile,
    state::{HaState, HaWorkerCtx},
    types::{
        last_start_success_at, last_success_at, wal_position, ApiVisibility, CoordinationAction,
        CoordinationState, DataDirState, DesiredState, DivergenceState, ElectionEligibility,
        GlobalKnowledge, IneligibleReason, LeadershipView, LeaseEpoch, LocalAction, LocalDataState,
        LocalKnowledge, ObservationState, ObservedPrimary, PeerKnowledge, PeerLeaderState,
        PostgresState, PrimaryObservation, ProcessState, PublicationGoal, PublicationState,
        ReconcilePlan, ReplicationState, StaleLeaseReason, StorageState, SwitchoverRequest,
        SwitchoverState, SwitchoverTarget, WalPosition, WorldView,
    },
};

pub(crate) async fn run(mut ctx: HaWorkerCtx) -> Result<(), WorkerError> {
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

pub(crate) async fn step_once(ctx: &mut HaWorkerCtx) -> Result<(), WorkerError> {
    let now = (ctx.cadence.now)()?;
    let world = observe(ctx, now)?;
    let desired = decide(&world, &ctx.identity.self_id);
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

fn observe(ctx: &HaWorkerCtx, now: crate::state::UnixMillis) -> Result<WorldView, WorkerError> {
    let config = ctx.observed.config.latest();
    let pg = ctx.observed.pg.latest();
    let dcs = ctx.observed.dcs.latest();
    let process = ctx.observed.process.latest();
    let data_dir_path = config.postgres.paths.data_dir.clone();
    let observed_primary = observed_primary_member(&dcs, &ctx.identity.self_id);
    let local_data_timeline = pg_timeline_id(&pg).or_else(|| {
        dcs.members
            .get(&ctx.identity.self_id)
            .and_then(member_timeline)
    });
    let local_system_identifier = pg_system_identifier(&pg).or_else(|| {
        dcs.members
            .get(&ctx.identity.self_id)
            .and_then(member_system_identifier)
    });

    let local = LocalKnowledge {
        data_dir: build_data_dir_state(
            data_dir_path.as_path(),
            local_data_timeline,
            local_system_identifier,
            &observed_primary,
        ),
        postgres: build_local_postgres_state(&pg, &dcs),
        process: ProcessState::from(&process),
        storage: build_storage_state(
            &dcs,
            &pg,
            config.ha.lease_ttl_ms,
            &ctx.identity.self_id,
            now,
        ),
        managed_roles_reconciled: ctx.state_channel.current.managed_roles_reconciled,
        publication: ctx.state_channel.current.publication.clone(),
        observation: ObservationState {
            pg_observed_at: pg.last_refresh_at().unwrap_or(now),
            last_start_success_at: last_start_success_at(&process),
            last_promote_success_at: last_success_at(&process, ActiveJobKind::Promote),
            last_demote_success_at: last_success_at(&process, ActiveJobKind::Demote),
        },
    };
    let global = build_global_knowledge(&dcs, &pg, &local.data_dir, &ctx.identity.self_id);

    Ok(WorldView { local, global })
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
    ctx: &mut HaWorkerCtx,
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
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &CoordinationAction,
) -> Result<(), WorkerError> {
    match action {
        CoordinationAction::AcquireLease(_kind) => ctx
            .control
            .dcs_handle
            .acquire_leadership()
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "ha acquire lease failed at tick {ha_tick} index {action_index}: {err}"
                ))
            }),
        CoordinationAction::ReleaseLease => ctx
            .control
            .dcs_handle
            .release_leadership()
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "ha release lease failed at tick {ha_tick} index {action_index}: {err}"
                ))
            }),
        CoordinationAction::ClearSwitchover => ctx
            .control
            .dcs_handle
            .clear_switchover()
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "ha clear switchover failed at tick {ha_tick} index {action_index}: {err}"
                ))
            }),
    }
}

async fn execute_local_action(
    ctx: &mut HaWorkerCtx,
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
    ctx: &mut HaWorkerCtx,
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
    local_timeline: Option<TimelineId>,
    local_system_identifier: Option<SystemIdentifier>,
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
            committed_lsn: *wal_lsn,
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
            timeline: timeline.unwrap_or(TimelineId::UNKNOWN),
            lsn: replay_lsn,
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
    let self_member = dcs.members.get(self_id);
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
    let leadership = build_leadership_view(dcs, self_id);
    let peers = dcs
        .members
        .iter()
        .filter(|(member_id, _)| *member_id != self_id)
        .map(|(member_id, member)| (member_id.clone(), build_peer_knowledge_from_member(member)))
        .collect();
    let primary = observed_primary_member(dcs, self_id)
        .map(PrimaryObservation::Observed)
        .unwrap_or(PrimaryObservation::Absent);

    GlobalKnowledge {
        coordination: CoordinationState {
            trust: dcs.trust.clone(),
            leadership,
            primary,
        },
        switchover: match &dcs.switchover {
            DcsSwitchoverStateView::None => SwitchoverState::None,
            DcsSwitchoverStateView::Requested(request) => {
                SwitchoverState::Requested(SwitchoverRequest {
                    target: match &request.target {
                        DcsSwitchoverTargetView::AnyHealthyReplica => {
                            SwitchoverTarget::AnyHealthyReplica
                        }
                        DcsSwitchoverTargetView::Specific(member_id) => {
                            SwitchoverTarget::Specific(member_id.clone())
                        }
                    },
                })
            }
        },
        peers,
        self_peer: build_self_peer(pg, local_data_dir),
    }
}

fn build_peer_knowledge_from_member(member: &DcsMemberView) -> PeerKnowledge {
    let api = if member.routing.api.is_some() {
        ApiVisibility::Reachable
    } else {
        ApiVisibility::Unreachable
    };
    let eligibility = if api == ApiVisibility::Unreachable {
        ElectionEligibility::Ineligible(IneligibleReason::ApiUnavailable)
    } else {
        match &member.postgres {
            DcsMemberPostgresView::Unknown(observation) => {
                if observation.readiness == Readiness::Ready {
                    ElectionEligibility::BootstrapEligible
                } else {
                    ElectionEligibility::Ineligible(IneligibleReason::NotReady)
                }
            }
            DcsMemberPostgresView::Primary(observation) => {
                if observation.readiness != Readiness::Ready {
                    return PeerKnowledge {
                        eligibility: ElectionEligibility::Ineligible(IneligibleReason::NotReady),
                        api,
                    };
                }
                wal_position(
                    observation.committed_wal.timeline,
                    Some(observation.committed_wal.lsn),
                )
                .map(ElectionEligibility::PromoteEligible)
                .unwrap_or(ElectionEligibility::Ineligible(IneligibleReason::Lagging))
            }
            DcsMemberPostgresView::Replica(observation) => {
                if observation.readiness != Readiness::Ready {
                    return PeerKnowledge {
                        eligibility: ElectionEligibility::Ineligible(IneligibleReason::NotReady),
                        api,
                    };
                }
                observation
                    .replay_wal
                    .as_ref()
                    .or(observation.follow_wal.as_ref())
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
            Some(committed_lsn),
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

    dcs.members.iter().find_map(|(member_id, member)| {
        let endpoint = &member.routing.postgres;
        (endpoint.host == primary_conninfo.host && endpoint.port == primary_conninfo.port)
            .then_some(member_id.clone())
    })
}

fn build_leadership_view(dcs: &DcsView, self_id: &MemberId) -> LeadershipView {
    match &dcs.leader {
        DcsLeaderStateView::Unheld => LeadershipView::Open,
        DcsLeaderStateView::Held(leader) => {
            let epoch = LeaseEpoch {
                holder: leader.holder.clone(),
                generation: leader.generation,
            };
            if epoch.holder == *self_id {
                return LeadershipView::HeldBySelf(epoch);
            }

            match dcs.members.get(&epoch.holder) {
                None => LeadershipView::StaleObservedLease {
                    epoch,
                    reason: StaleLeaseReason::HolderMissing,
                },
                Some(member) => classify_foreign_leader(member, epoch),
            }
        }
    }
}

fn classify_foreign_leader(member: &DcsMemberView, epoch: LeaseEpoch) -> LeadershipView {
    match &member.postgres {
        DcsMemberPostgresView::Primary(observation)
            if observation.readiness == Readiness::Ready =>
        {
            LeadershipView::HeldByPeer {
                epoch,
                state: PeerLeaderState::PrimaryReady,
            }
        }
        DcsMemberPostgresView::Primary(_) => LeadershipView::HeldByPeer {
            epoch,
            state: PeerLeaderState::Recovering,
        },
        DcsMemberPostgresView::Unknown(observation)
            if observation.readiness == Readiness::Ready =>
        {
            LeadershipView::HeldByPeer {
                epoch,
                state: PeerLeaderState::Unreachable,
            }
        }
        DcsMemberPostgresView::Unknown(_) => LeadershipView::StaleObservedLease {
            epoch,
            reason: StaleLeaseReason::HolderNotReady,
        },
        DcsMemberPostgresView::Replica(_) => LeadershipView::StaleObservedLease {
            epoch,
            reason: StaleLeaseReason::HolderNotPrimary,
        },
    }
}

fn observed_primary_member(dcs: &DcsView, self_id: &MemberId) -> Option<ObservedPrimary> {
    dcs.members
        .values()
        .find(|member| {
            member.member_id != *self_id
                && matches!(
                    &member.postgres,
                    DcsMemberPostgresView::Primary(observation) if observation.readiness == Readiness::Ready
                )
        })
        .map(|member| ObservedPrimary {
            member: member.member_id.clone(),
            timeline: member_timeline(member),
            system_identifier: member_system_identifier(member),
        })
}

fn member_timeline(member: &DcsMemberView) -> Option<TimelineId> {
    match &member.postgres {
        DcsMemberPostgresView::Unknown(observation) => observation.timeline,
        DcsMemberPostgresView::Primary(observation) => observation.committed_wal.timeline,
        DcsMemberPostgresView::Replica(observation) => observation
            .replay_wal
            .as_ref()
            .and_then(|value| value.timeline)
            .or_else(|| {
                observation
                    .follow_wal
                    .as_ref()
                    .and_then(|value| value.timeline)
            }),
    }
}

fn member_system_identifier(member: &DcsMemberView) -> Option<SystemIdentifier> {
    match &member.postgres {
        DcsMemberPostgresView::Unknown(observation) => observation.system_identifier,
        DcsMemberPostgresView::Primary(observation) => observation.system_identifier,
        DcsMemberPostgresView::Replica(observation) => observation.system_identifier,
    }
}

fn pg_system_identifier(pg: &PgInfoState) -> Option<SystemIdentifier> {
    match pg {
        PgInfoState::Unknown { common }
        | PgInfoState::Primary { common, .. }
        | PgInfoState::Replica { common, .. } => common.system_identifier,
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
        dcs::{
            DcsLeaderStateView, DcsMemberEndpointView, DcsMemberLeaseView, DcsMemberRoutingView,
            DcsMemberView, DcsSwitchoverStateView, DcsTrust, DcsUnknownPostgresView, DcsView,
        },
        pginfo::state::PgConnInfo,
    };
    use crate::{
        pginfo::state::{PgConfig, PgInfoCommon, Readiness, SqlStatus},
        state::{SystemIdentifier, TimelineId, UnixMillis, WalLsn, WorkerStatus},
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

    fn replica_pg_state_with_primary_conninfo(host: &str, port: u16) -> PgInfoState {
        let mut state = replica_pg_state(67_272_104, Some(67_272_104));
        if let PgInfoState::Replica { common, .. } = &mut state {
            common.pg_config.primary_conninfo = Some(PgConnInfo {
                host: host.to_string(),
                port,
                user: "replicator".to_string(),
                dbname: "postgres".to_string(),
                application_name: Some("node-a".to_string()),
                connect_timeout_s: Some(5),
                ssl_mode: crate::pginfo::state::PgSslMode::Require,
                ssl_root_cert: None,
                options: None,
            });
        }
        state
    }

    fn dcs_view_for_member(member_id: &str, host: &str, port: u16) -> DcsView {
        DcsView {
            worker: WorkerStatus::Running,
            trust: DcsTrust::FullQuorum,
            members: BTreeMap::from([(
                MemberId(member_id.to_string()),
                DcsMemberView {
                    member_id: MemberId(member_id.to_string()),
                    lease: DcsMemberLeaseView { ttl_ms: 10_000 },
                    routing: DcsMemberRoutingView {
                        postgres: DcsMemberEndpointView {
                            host: host.to_string(),
                            port,
                        },
                        api: None,
                    },
                    postgres: DcsMemberPostgresView::Unknown(DcsUnknownPostgresView {
                        readiness: Readiness::Ready,
                        timeline: Some(TimelineId(7)),
                        system_identifier: Some(SystemIdentifier(41)),
                    }),
                },
            )]),
            leader: DcsLeaderStateView::Unheld,
            switchover: DcsSwitchoverStateView::None,
            last_observed_at: Some(UnixMillis(123)),
        }
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
                timeline: TimelineId(7),
                lsn: WalLsn(67_272_104),
            })
        );
    }

    #[test]
    fn local_postgres_state_resolves_replica_upstream_from_primary_conninfo() {
        let state = build_local_postgres_state(
            &replica_pg_state_with_primary_conninfo("node-b", 5432),
            &dcs_view_for_member("node-b", "node-b", 5432),
        );

        assert_eq!(
            state,
            PostgresState::Replica {
                upstream: Some(MemberId("node-b".to_string())),
                replication: ReplicationState::Streaming(WalPosition {
                    timeline: TimelineId(7),
                    lsn: WalLsn(67_272_104),
                }),
            }
        );
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
            Some(TimelineId(7)),
            Some(SystemIdentifier(41)),
            &Some(ObservedPrimary {
                member: MemberId("node-c".to_string()),
                timeline: Some(TimelineId(8)),
                system_identifier: Some(SystemIdentifier(99)),
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
}
