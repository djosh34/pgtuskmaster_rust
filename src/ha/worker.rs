use std::path::Path;

use crate::{
    dcs::{state::MemberRole, store::DcsStoreError},
    postgres_roles,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    process::jobs::ActiveJobKind,
    state::{MemberId, WorkerError, WorkerStatus},
};

use super::{
    decide::decide,
    process_dispatch::{dispatch_process_action, ProcessDispatchError},
    reconcile::reconcile,
    state::{HaState, HaWorkerCtx},
    types::{
        last_success_at, wal_position, ApiVisibility, AuthorityView, DataDirState, DesiredState,
        DivergenceState, ElectionEligibility, GlobalKnowledge, IneligibleReason, LeaseEpoch,
        LeaseState, LocalDataState, LocalKnowledge, ObservationState, PeerKnowledge, PostgresState,
        ProcessState, PublicationGoal, PublicationState, ReconcileAction, ReplicationState,
        StorageState, SwitchoverRequest, SwitchoverState, SwitchoverTarget, WalPosition, WorldView,
    },
};

pub(crate) async fn run(mut ctx: HaWorkerCtx) -> Result<(), WorkerError> {
    let mut interval = tokio::time::interval(ctx.poll_interval);
    loop {
        tokio::select! {
            changed = ctx.pg_subscriber.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha pg subscriber closed: {err}")))?;
            }
            changed = ctx.dcs_subscriber.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha dcs subscriber closed: {err}")))?;
            }
            changed = ctx.process_subscriber.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha process subscriber closed: {err}")))?;
            }
            changed = ctx.config_subscriber.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha config subscriber closed: {err}")))?;
            }
            _ = interval.tick() => {}
        }
        step_once(&mut ctx).await?;
    }
}

pub(crate) async fn step_once(ctx: &mut HaWorkerCtx) -> Result<(), WorkerError> {
    let now = (ctx.now)()?;
    let world = observe(ctx, now)?;
    let desired = decide(&world, &ctx.self_id);
    let actions = reconcile(&world, &desired);
    let next_state = build_next_state(&ctx.state, &desired, &actions);

    ctx.publisher
        .publish(next_state.clone(), now)
        .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;
    ctx.state = next_state;

    for (action_index, action) in actions.iter().enumerate() {
        execute_action(ctx, ctx.state.tick, action_index, action).await?;
    }

    Ok(())
}

fn observe(ctx: &HaWorkerCtx, now: crate::state::UnixMillis) -> Result<WorldView, WorkerError> {
    let config = ctx.config_subscriber.latest();
    let pg = ctx.pg_subscriber.latest();
    let dcs = ctx.dcs_subscriber.latest();
    let process = ctx.process_subscriber.latest();
    let data_dir_path = config.value.postgres.data_dir.clone();
    let observed_primary = observed_primary_member(&dcs.value, &ctx.self_id);
    let local_data_timeline = pg_timeline(&pg.value).or_else(|| {
        dcs.value
            .cache
            .members
            .get(&ctx.self_id)
            .and_then(|member| member.timeline.map(|timeline| u64::from(timeline.0)))
    });

    let local = LocalKnowledge {
        data_dir: build_data_dir_state(
            data_dir_path.as_path(),
            local_data_timeline,
            &observed_primary,
        ),
        postgres: build_postgres_state(&pg.value),
        process: ProcessState::from(&process.value),
        storage: build_storage_state(&dcs.value, &pg, &ctx.self_id, now),
        required_roles_ready: ctx.state.required_roles_ready,
        publication: ctx.state.publication.clone(),
        observation: ObservationState {
            pg_observed_at: pg.updated_at,
            last_start_success_at: last_success_at(&process.value, ActiveJobKind::StartPostgres),
            last_promote_success_at: last_success_at(&process.value, ActiveJobKind::Promote),
            last_demote_success_at: last_success_at(&process.value, ActiveJobKind::Demote),
        },
    };
    let global = build_global_knowledge(&dcs.value, &pg.value, &local.data_dir, &ctx.self_id);

    Ok(WorldView { local, global })
}

fn build_next_state(
    current: &HaState,
    desired: &DesiredState,
    actions: &[ReconcileAction],
) -> HaState {
    HaState {
        worker: WorkerStatus::Running,
        tick: current.tick.saturating_add(1),
        required_roles_ready: next_required_roles_ready(current, actions),
        publication: apply_publication_goal(&current.publication, &desired.publication),
        role: desired.role.clone(),
        clear_switchover: desired.clear_switchover,
        planned_actions: actions.to_vec(),
    }
}

fn next_required_roles_ready(current: &HaState, actions: &[ReconcileAction]) -> bool {
    if actions.iter().any(|action| {
        matches!(
            action,
            ReconcileAction::InitDb
                | ReconcileAction::BaseBackup(_)
                | ReconcileAction::StartReplica(_)
        )
    }) {
        return false;
    }

    current.required_roles_ready
}

fn apply_publication_goal(current: &PublicationState, goal: &PublicationGoal) -> PublicationState {
    match goal {
        PublicationGoal::KeepCurrent => current.clone(),
        PublicationGoal::PublishPrimary { primary, epoch } => PublicationState {
            authority: AuthorityView::Primary {
                member: primary.clone(),
                epoch: epoch.clone(),
            },
            fence_cutoff: None,
        },
        PublicationGoal::PublishNoPrimary {
            reason,
            fence_cutoff,
        } => PublicationState {
            authority: AuthorityView::NoPrimary(reason.clone()),
            fence_cutoff: fence_cutoff.clone(),
        },
    }
}

async fn execute_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &ReconcileAction,
) -> Result<(), WorkerError> {
    match action {
        ReconcileAction::AcquireLease(_kind) => {
            match ctx.dcs_store.acquire_leader_lease(&ctx.scope, &ctx.self_id) {
                Ok(()) | Err(DcsStoreError::AlreadyExists(_)) => Ok(()),
                Err(err) => Err(WorkerError::Message(format!(
                    "ha acquire lease failed at tick {ha_tick} index {action_index}: {err}"
                ))),
            }
        }
        ReconcileAction::ReleaseLease => {
            match ctx.dcs_store.release_leader_lease(&ctx.scope, &ctx.self_id) {
                Ok(()) | Err(DcsStoreError::LeaderLeaseNotOwned(_)) => Ok(()),
                Err(err) => Err(WorkerError::Message(format!(
                    "ha release lease failed at tick {ha_tick} index {action_index}: {err}"
                ))),
            }
        }
        ReconcileAction::ClearSwitchover => {
            ctx.dcs_store.clear_switchover(&ctx.scope).map_err(|err| {
                WorkerError::Message(format!(
                    "ha clear switchover failed at tick {ha_tick} index {action_index}: {err}"
                ))
            })
        }
        ReconcileAction::EnsureRequiredRoles => {
            let runtime_config = ctx.config_subscriber.latest().value;
            postgres_roles::ensure_required_roles(
                &runtime_config,
                ctx.process_defaults.socket_dir.as_path(),
                ctx.process_defaults.postgres_port,
            )
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "ha ensure required roles failed at tick {ha_tick} index {action_index}: {err}"
                ))
            })?;
            ctx.state.required_roles_ready = true;
            Ok(())
        }
        ReconcileAction::Publish(_) => Ok(()),
        process_action => {
            let runtime_config = ctx.config_subscriber.latest().value;
            dispatch_process_action(ctx, ha_tick, action_index, process_action, &runtime_config)
                .map(|_| ())
                .map_err(|err| map_process_dispatch_error(ha_tick, action_index, err))
        }
    }
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
    observed_primary: &Option<(MemberId, Option<u64>)>,
) -> DataDirState {
    let pg_version_path = data_dir.join("PG_VERSION");
    if !data_dir.exists() {
        return DataDirState::Missing;
    }
    if !pg_version_path.exists() {
        return DataDirState::Initialized(LocalDataState::BootstrapEmpty);
    }

    let local_state = match observed_primary {
        Some((_leader_member_id, leader_timeline)) if leader_timeline == &local_timeline => {
            LocalDataState::ConsistentReplica
        }
        Some((_leader_member_id, Some(_))) if local_timeline.is_some() => {
            LocalDataState::Diverged(DivergenceState::RewindPossible)
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
    dcs: &crate::dcs::state::DcsState,
    pg: &crate::state::Versioned<PgInfoState>,
    self_id: &MemberId,
    now: crate::state::UnixMillis,
) -> StorageState {
    let self_member = dcs.cache.members.get(self_id);
    let pg_observation_stale =
        now.0.saturating_sub(pg.updated_at.0) > dcs.cache.config.ha.lease_ttl_ms;
    if matches!(
        build_postgres_state(&pg.value),
        PostgresState::Primary { .. }
    ) && (self_member.is_none() || pg_observation_stale)
    {
        StorageState::Stalled
    } else {
        StorageState::Healthy
    }
}

fn build_global_knowledge(
    dcs: &crate::dcs::state::DcsState,
    pg: &PgInfoState,
    local_data_dir: &DataDirState,
    self_id: &MemberId,
) -> GlobalKnowledge {
    let observed_lease = dcs.cache.leader.as_ref().map(|leader| LeaseEpoch {
        holder: leader.member_id.clone(),
        generation: leader.generation,
    });
    let lease = observed_lease
        .as_ref()
        .map(|epoch| {
            if epoch.holder == *self_id {
                LeaseState::HeldByMe(epoch.clone())
            } else if leader_is_available(dcs, &epoch.holder) {
                LeaseState::HeldByPeer(epoch.clone())
            } else {
                LeaseState::Unheld
            }
        })
        .unwrap_or(LeaseState::Unheld);
    let peers = dcs
        .cache
        .members
        .iter()
        .filter(|(member_id, _)| *member_id != self_id)
        .map(|(member_id, member)| (member_id.clone(), build_peer_knowledge_from_member(member)))
        .collect();

    GlobalKnowledge {
        dcs_trust: dcs.trust.clone(),
        lease,
        observed_lease,
        observed_primary: observed_primary_member(dcs, self_id).map(|(member_id, _)| member_id),
        switchover: dcs
            .cache
            .switchover
            .as_ref()
            .map(|request| {
                SwitchoverState::Requested(SwitchoverRequest {
                    target: request
                        .switchover_to
                        .as_ref()
                        .map_or(SwitchoverTarget::AnyHealthyReplica, |member_id| {
                            SwitchoverTarget::Specific(member_id.clone())
                        }),
                })
            })
            .unwrap_or(SwitchoverState::None),
        peers,
        self_peer: build_self_peer(pg, local_data_dir),
    }
}

fn build_peer_knowledge_from_member(member: &crate::dcs::state::MemberRecord) -> PeerKnowledge {
    let api = if member.api_url.is_some() {
        ApiVisibility::Reachable
    } else {
        ApiVisibility::Unreachable
    };
    let election = if api == ApiVisibility::Unreachable {
        ElectionEligibility::Ineligible(IneligibleReason::ApiUnavailable)
    } else if member.sql != SqlStatus::Healthy || member.readiness != Readiness::Ready {
        ElectionEligibility::Ineligible(IneligibleReason::NotReady)
    } else {
        match member.role {
            MemberRole::Primary => wal_position(member.timeline, member.write_lsn)
                .map(ElectionEligibility::PromoteEligible)
                .unwrap_or(ElectionEligibility::Ineligible(IneligibleReason::Lagging)),
            MemberRole::Replica => wal_position(member.timeline, member.replay_lsn)
                .map(ElectionEligibility::PromoteEligible)
                .unwrap_or(ElectionEligibility::Ineligible(IneligibleReason::Lagging)),
            MemberRole::Unknown => ElectionEligibility::BootstrapEligible,
        }
    };

    PeerKnowledge { election, api }
}

fn build_self_peer(pg: &PgInfoState, local_data_dir: &DataDirState) -> PeerKnowledge {
    let election = match (local_data_dir, build_postgres_state(pg)) {
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
        (_, PostgresState::Replica { replication, .. }) => match replication {
            ReplicationState::Streaming(position) | ReplicationState::CatchingUp(position) => {
                ElectionEligibility::PromoteEligible(position)
            }
            ReplicationState::Stalled => ElectionEligibility::Ineligible(IneligibleReason::Lagging),
        },
        (_, PostgresState::Offline) => {
            ElectionEligibility::Ineligible(IneligibleReason::StartingUp)
        }
    };
    PeerKnowledge {
        election,
        api: ApiVisibility::Reachable,
    }
}

fn leader_is_available(dcs: &crate::dcs::state::DcsState, leader_member_id: &MemberId) -> bool {
    dcs.cache
        .members
        .get(leader_member_id)
        .is_some_and(|member| {
            member.role == MemberRole::Primary
                && member.sql == SqlStatus::Healthy
                && member.readiness == Readiness::Ready
        })
}

fn observed_primary_member(
    dcs: &crate::dcs::state::DcsState,
    self_id: &MemberId,
) -> Option<(MemberId, Option<u64>)> {
    dcs.cache
        .members
        .values()
        .find(|member| {
            member.member_id != *self_id
                && member.role == MemberRole::Primary
                && member.sql == SqlStatus::Healthy
                && member.readiness == Readiness::Ready
        })
        .map(|member| {
            (
                member.member_id.clone(),
                member.timeline.map(|timeline| u64::from(timeline.0)),
            )
        })
}

fn pg_timeline(pg: &PgInfoState) -> Option<u64> {
    pg_timeline_id(pg).map(|timeline| u64::from(timeline.0))
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
    use std::{fs, path::PathBuf};

    use crate::{
        config::RuntimeConfig,
        dcs::state::{DcsCache, DcsState, DcsTrust, MemberRecord, MemberRole},
        ha::types::{DataDirState, DivergenceState, LocalDataState},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        state::{MemberId, TimelineId, UnixMillis, Version, Versioned, WorkerStatus},
    };

    use super::{build_data_dir_state, build_storage_state, StorageState};

    fn temp_data_dir(label: &str) -> Result<PathBuf, String> {
        let current_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|err| format!("read current time failed: {err}"))?
            .as_millis();
        let unique = format!("{label}-{}-{}", std::process::id(), current_millis);
        Ok(std::env::temp_dir().join(format!("pgtuskmaster-ha-worker-{unique}")))
    }

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_primary_pg(updated_at: UnixMillis) -> Versioned<PgInfoState> {
        Versioned::new(
            Version(1),
            updated_at,
            PgInfoState::Primary {
                common: PgInfoCommon {
                    worker: WorkerStatus::Running,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: Some(TimelineId(1)),
                    pg_config: PgConfig {
                        port: Some(5432),
                        hot_standby: Some(false),
                        primary_conninfo: None,
                        primary_slot_name: None,
                        extra: std::collections::BTreeMap::new(),
                    },
                    last_refresh_at: Some(updated_at),
                },
                wal_lsn: crate::state::WalLsn(42),
                slots: Vec::new(),
            },
        )
    }

    fn sample_replica_pg(updated_at: UnixMillis) -> Versioned<PgInfoState> {
        Versioned::new(
            Version(1),
            updated_at,
            PgInfoState::Replica {
                common: PgInfoCommon {
                    worker: WorkerStatus::Running,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: Some(TimelineId(1)),
                    pg_config: PgConfig {
                        port: Some(5432),
                        hot_standby: Some(true),
                        primary_conninfo: None,
                        primary_slot_name: None,
                        extra: std::collections::BTreeMap::new(),
                    },
                    last_refresh_at: Some(updated_at),
                },
                replay_lsn: crate::state::WalLsn(42),
                follow_lsn: Some(crate::state::WalLsn(42)),
                upstream: None,
            },
        )
    }

    fn sample_dcs_state() -> DcsState {
        let cfg = sample_runtime_config();
        DcsState {
            worker: WorkerStatus::Running,
            trust: DcsTrust::FullQuorum,
            cache: DcsCache {
                members: std::collections::BTreeMap::from([(
                    MemberId("node-a".to_string()),
                    MemberRecord {
                        member_id: MemberId("node-a".to_string()),
                        postgres_host: "127.0.0.1".to_string(),
                        postgres_port: 5432,
                        api_url: Some("https://node-a:8443".to_string()),
                        role: MemberRole::Primary,
                        sql: SqlStatus::Healthy,
                        readiness: Readiness::Ready,
                        timeline: Some(TimelineId(1)),
                        write_lsn: Some(crate::state::WalLsn(42)),
                        replay_lsn: None,
                        pg_version: Version(1),
                    },
                )]),
                leader: None,
                switchover: None,
                config: cfg,
                init_lock: None,
            },
            last_refresh_at: Some(UnixMillis(100)),
        }
    }

    #[test]
    fn build_data_dir_state_uses_fallback_local_timeline_for_divergence() -> Result<(), String> {
        let data_dir = temp_data_dir("fallback-timeline")?;
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir failed for {}: {err}", data_dir.display()))?;
        fs::write(data_dir.join("PG_VERSION"), "16")
            .map_err(|err| format!("write PG_VERSION failed for {}: {err}", data_dir.display()))?;

        let result = build_data_dir_state(
            data_dir.as_path(),
            Some(1),
            &Some((MemberId("node-a".to_string()), Some(2))),
        );

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove data dir failed for {}: {err}", data_dir.display()))?;

        assert_eq!(
            result,
            DataDirState::Initialized(LocalDataState::Diverged(DivergenceState::RewindPossible))
        );
        Ok(())
    }

    #[test]
    fn build_data_dir_state_keeps_consistent_replica_without_local_timeline() -> Result<(), String>
    {
        let data_dir = temp_data_dir("no-fallback-timeline")?;
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir failed for {}: {err}", data_dir.display()))?;
        fs::write(data_dir.join("PG_VERSION"), "16")
            .map_err(|err| format!("write PG_VERSION failed for {}: {err}", data_dir.display()))?;

        let result = build_data_dir_state(
            data_dir.as_path(),
            None,
            &Some((MemberId("node-a".to_string()), Some(2))),
        );

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove data dir failed for {}: {err}", data_dir.display()))?;

        assert_eq!(
            result,
            DataDirState::Initialized(LocalDataState::ConsistentReplica)
        );
        Ok(())
    }

    #[test]
    fn build_storage_state_marks_primary_as_stalled_when_pg_observation_is_stale() {
        let dcs = sample_dcs_state();
        let pg = sample_primary_pg(UnixMillis(1));

        let result = build_storage_state(
            &dcs,
            &pg,
            &MemberId("node-a".to_string()),
            UnixMillis(20_000),
        );

        assert_eq!(result, StorageState::Stalled);
    }

    #[test]
    fn build_storage_state_keeps_replica_healthy_when_pg_observation_is_stale() {
        let dcs = sample_dcs_state();
        let pg = sample_replica_pg(UnixMillis(1));

        let result = build_storage_state(
            &dcs,
            &pg,
            &MemberId("node-a".to_string()),
            UnixMillis(20_000),
        );

        assert_eq!(result, StorageState::Healthy);
    }
}
