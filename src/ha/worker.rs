use std::path::Path;

use crate::{
    dcs::{
        DcsLeaderStateView, DcsMemberPostgresView, DcsMemberView, DcsSwitchoverStateView,
        DcsSwitchoverTargetView, DcsView,
    },
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    postgres_roles,
    process::jobs::ActiveJobKind,
    state::{MemberId, WorkerError, WorkerStatus},
};

use super::{
    decide::decide,
    process_dispatch::{dispatch_process_action, ProcessDispatchError},
    reconcile::reconcile,
    state::{HaState, HaWorkerCtx},
    types::{
        last_success_at, wal_position, ApiVisibility, DataDirState, DesiredState, DivergenceState,
        ElectionEligibility, GlobalKnowledge, IneligibleReason, LeaseEpoch, LeaseState,
        LocalDataState, LocalKnowledge, ObservationState, PeerKnowledge, PostgresState,
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
    let next_state = build_next_state(&ctx.state, &world, &desired, &actions);

    ctx.publisher
        .publish(next_state.clone())
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
    let data_dir_path = config.postgres.data_dir.clone();
    let observed_primary = observed_primary_member(&dcs, &ctx.self_id);
    let local_data_timeline = pg_timeline(&pg).or_else(|| {
        dcs.members.get(&ctx.self_id).and_then(member_timeline)
    });

    let local = LocalKnowledge {
        data_dir: build_data_dir_state(
            data_dir_path.as_path(),
            local_data_timeline,
            &observed_primary,
        ),
        postgres: build_postgres_state(&pg),
        process: ProcessState::from(&process),
        storage: build_storage_state(&dcs, &pg, config.ha.lease_ttl_ms, &ctx.self_id, now),
        required_roles_ready: ctx.state.required_roles_ready,
        publication: ctx.state.publication.clone(),
        observation: ObservationState {
            pg_observed_at: pg.last_refresh_at().unwrap_or(now),
            last_start_success_at: last_success_at(&process, ActiveJobKind::StartPostgres),
            last_promote_success_at: last_success_at(&process, ActiveJobKind::Promote),
            last_demote_success_at: last_success_at(&process, ActiveJobKind::Demote),
        },
    };
    let global = build_global_knowledge(&dcs, &pg, &local.data_dir, &ctx.self_id);

    Ok(WorldView { local, global })
}

fn build_next_state(
    current: &HaState,
    world: &WorldView,
    desired: &DesiredState,
    actions: &[ReconcileAction],
) -> HaState {
    HaState {
        worker: WorkerStatus::Running,
        tick: current.tick.saturating_add(1),
        required_roles_ready: next_required_roles_ready(current, actions),
        publication: apply_publication_goal(&current.publication, &desired.publication),
        role: desired.role.clone(),
        world: world.clone(),
        clear_switchover: desired.clear_switchover,
        planned_commands: actions.to_vec(),
    }
}

fn next_required_roles_ready(current: &HaState, actions: &[ReconcileAction]) -> bool {
    if actions.iter().any(|action| {
        matches!(
            action,
            ReconcileAction::InitDb
                | ReconcileAction::BaseBackup(_)
                | ReconcileAction::StartDetachedStandby
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
            authority: super::types::AuthorityView::Primary {
                member: primary.clone(),
                epoch: epoch.clone(),
            },
            fence_cutoff: None,
        },
        PublicationGoal::PublishNoPrimary {
            reason,
            fence_cutoff,
        } => PublicationState {
            authority: super::types::AuthorityView::NoPrimary(reason.clone()),
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
        ReconcileAction::AcquireLease(_kind) => ctx
            .dcs_handle
            .acquire_leadership()
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "ha acquire lease failed at tick {ha_tick} index {action_index}: {err}"
                ))
            }),
        ReconcileAction::ReleaseLease => ctx
            .dcs_handle
            .release_leadership()
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "ha release lease failed at tick {ha_tick} index {action_index}: {err}"
                ))
            }),
        ReconcileAction::ClearSwitchover => ctx.dcs_handle.clear_switchover().await.map_err(
            |err| {
                WorkerError::Message(format!(
                    "ha clear switchover failed at tick {ha_tick} index {action_index}: {err}"
                ))
            },
        ),
        ReconcileAction::EnsureRequiredRoles => {
            let runtime_config = ctx.config_subscriber.latest();
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
            let runtime_config = ctx.config_subscriber.latest();
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
    if matches!(build_postgres_state(pg), PostgresState::Primary { .. })
        && (self_member.is_none() || pg_observation_stale)
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
    let observed_lease = match &dcs.leader {
        DcsLeaderStateView::Unheld => None,
        DcsLeaderStateView::Held(leader) => Some(LeaseEpoch {
            holder: leader.holder.clone(),
            generation: leader.generation,
        }),
    };
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
        .members
        .iter()
        .filter(|(member_id, _)| *member_id != self_id)
        .map(|(member_id, member)| (member_id.clone(), build_peer_knowledge_from_member(member)))
        .collect();

    GlobalKnowledge {
        dcs_trust: dcs.trust.clone(),
        lease: lease.clone(),
        observed_lease,
        observed_primary: observed_primary_member(dcs, self_id)
            .map(|(member_id, _)| member_id.clone()),
        coordination: super::types::CoordinationView {
            trust: dcs.trust.clone(),
            leader: lease,
            sampled_primary: observed_primary_member(dcs, self_id).map(|(member_id, _)| member_id),
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
        eligibility,
        api: ApiVisibility::Reachable,
    }
}

fn leader_is_available(dcs: &DcsView, leader_member_id: &MemberId) -> bool {
    dcs.members
        .get(leader_member_id)
        .is_some_and(|member| {
            matches!(
                &member.postgres,
                DcsMemberPostgresView::Primary(observation) if observation.readiness == Readiness::Ready
            )
        })
}

fn observed_primary_member(
    dcs: &DcsView,
    self_id: &MemberId,
) -> Option<(MemberId, Option<u64>)> {
    dcs.members
        .values()
        .find(|member| {
            member.member_id != *self_id
                && matches!(
                    &member.postgres,
                    DcsMemberPostgresView::Primary(observation) if observation.readiness == Readiness::Ready
                )
        })
        .map(|member| (member.member_id.clone(), member_timeline(member)))
}

fn member_timeline(member: &DcsMemberView) -> Option<u64> {
    match &member.postgres {
        DcsMemberPostgresView::Unknown(observation) => {
            observation.timeline.map(|value| u64::from(value.0))
        }
        DcsMemberPostgresView::Primary(observation) => observation
            .committed_wal
            .timeline
            .map(|value| u64::from(value.0)),
        DcsMemberPostgresView::Replica(observation) => observation
            .replay_wal
            .as_ref()
            .and_then(|value| value.timeline.map(|timeline| u64::from(timeline.0)))
            .or_else(|| {
                observation
                    .follow_wal
                    .as_ref()
                    .and_then(|value| value.timeline.map(|timeline| u64::from(timeline.0)))
            }),
    }
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
