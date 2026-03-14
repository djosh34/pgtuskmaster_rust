use super::types::{
    DataDirState, DesiredState, FailSafeGoal, FenceReason, FollowGoal, IdleReason,
    LeadershipView, LocalDataState, PostgresState, ProcessState, PublicationGoal,
    PublicationState, ReconcileAction, RecoveryPlan, TargetRole, WorldView,
};

pub(crate) fn reconcile(world: &WorldView, desired: &DesiredState) -> Vec<ReconcileAction> {
    let publication_actions = reconcile_publication(&world.local.publication, desired);
    let switchover_actions = reconcile_switchover(world, desired);
    let role_action = match &world.local.process {
        ProcessState::Running(_) => None,
        ProcessState::Idle | ProcessState::Failed(_) => reconcile_role(world, &desired.role),
    };

    publication_actions
        .into_iter()
        .chain(switchover_actions)
        .chain(role_action)
        .collect()
}

fn reconcile_publication(
    current: &PublicationState,
    desired: &DesiredState,
) -> Vec<ReconcileAction> {
    let publish_action = match &desired.publication {
        PublicationGoal::KeepCurrent => None,
        PublicationGoal::Publish(projection)
            if current == &PublicationState::Projected(projection.clone()) =>
        {
            None
        }
        publication => Some(ReconcileAction::Publish(publication.clone())),
    };

    publish_action.into_iter().collect()
}

fn reconcile_switchover(world: &WorldView, desired: &DesiredState) -> Vec<ReconcileAction> {
    match (&world.global.switchover, desired.clear_switchover) {
        (super::types::SwitchoverState::Requested(_), true) => {
            vec![ReconcileAction::ClearSwitchover]
        }
        (super::types::SwitchoverState::None, _) | (_, false) => Vec::new(),
    }
}

fn reconcile_role(world: &WorldView, target: &TargetRole) -> Option<ReconcileAction> {
    match target {
        TargetRole::Leader(_) => match (&world.local.data_dir, &world.local.postgres) {
            (DataDirState::Missing, _) => Some(ReconcileAction::InitDb),
            (DataDirState::Initialized(LocalDataState::BootstrapEmpty), _) => {
                Some(ReconcileAction::InitDb)
            }
            (_, _) if world.local.observation.waiting_for_fresh_pg_after_start() => None,
            (_, _) if world.local.observation.waiting_for_fresh_pg_after_promote() => None,
            (DataDirState::Initialized(_), PostgresState::Offline) => {
                Some(ReconcileAction::StartPrimary)
            }
            (DataDirState::Initialized(_), PostgresState::Replica { .. }) => {
                Some(ReconcileAction::Promote)
            }
            (DataDirState::Initialized(_), PostgresState::Primary { .. }) => {
                (!world.local.required_roles_ready).then_some(ReconcileAction::EnsureRequiredRoles)
            }
        },
        TargetRole::Candidate(kind) => Some(ReconcileAction::AcquireLease(kind.clone())),
        TargetRole::Follower(goal) => reconcile_follow_role(world, goal),
        TargetRole::FailSafe(goal) => reconcile_failsafe_role(world, goal),
        TargetRole::DemotingForSwitchover(_) => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                None
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast))
            }
            PostgresState::Offline => leadership_held_by_self(world)
                .then_some(ReconcileAction::ReleaseLease),
        },
        TargetRole::Fenced(reason) => reconcile_fenced_role(world, reason),
        TargetRole::Idle(reason) => reconcile_idle_role(world, reason),
    }
}

fn reconcile_follow_role(world: &WorldView, goal: &FollowGoal) -> Option<ReconcileAction> {
    match goal.recovery {
        RecoveryPlan::None => None,
        RecoveryPlan::Basebackup => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                None
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast))
            }
            PostgresState::Offline => Some(ReconcileAction::BaseBackup(goal.leader.clone())),
        },
        RecoveryPlan::Rewind => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                None
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast))
            }
            PostgresState::Offline => Some(ReconcileAction::PgRewind(goal.leader.clone())),
        },
        RecoveryPlan::StartStreaming => {
            if world.local.observation.waiting_for_fresh_pg_after_start() {
                return None;
            }
            if world.local.observation.waiting_for_fresh_pg_after_demote() {
                return None;
            }

            match &world.local.postgres {
                PostgresState::Offline => Some(ReconcileAction::StartReplica(goal.leader.clone())),
                PostgresState::Primary { .. } => {
                    Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast))
                }
                PostgresState::Replica {
                    upstream,
                    replication: _,
                } => match upstream {
                    Some(current_upstream) if current_upstream == &goal.leader => None,
                    Some(_) => Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast)),
                    None => Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast)),
                },
            }
        }
    }
}

fn reconcile_failsafe_role(world: &WorldView, goal: &FailSafeGoal) -> Option<ReconcileAction> {
    match goal {
        FailSafeGoal::PrimaryMustStop(_) => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                None
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => Some(
                ReconcileAction::Demote(super::types::ShutdownMode::Immediate),
            ),
            PostgresState::Offline => None,
        },
        FailSafeGoal::ReplicaKeepFollowing(_) => None,
        FailSafeGoal::WaitForQuorum => match &world.local.postgres {
            PostgresState::Primary { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                None
            }
            PostgresState::Primary { .. } => Some(ReconcileAction::Demote(
                super::types::ShutdownMode::Immediate,
            )),
            PostgresState::Replica { .. } => None,
            PostgresState::Offline => None,
        },
    }
}

fn reconcile_fenced_role(world: &WorldView, reason: &FenceReason) -> Option<ReconcileAction> {
    match reason {
        FenceReason::StorageStalled if leadership_held_by_self(world) => {
            Some(ReconcileAction::ReleaseLease)
        }
        FenceReason::ForeignLeaderDetected | FenceReason::StorageStalled => {
            match &world.local.postgres {
                PostgresState::Primary { .. } | PostgresState::Replica { .. }
                    if world.local.observation.waiting_for_fresh_pg_after_demote() =>
                {
                    None
                }
                PostgresState::Primary { .. } | PostgresState::Replica { .. } => Some(
                    ReconcileAction::Demote(super::types::ShutdownMode::Immediate),
                ),
                PostgresState::Offline => leadership_held_by_self(world)
                    .then_some(ReconcileAction::ReleaseLease),
            }
        }
    }
}

fn reconcile_idle_role(world: &WorldView, _reason: &IdleReason) -> Option<ReconcileAction> {
    match &world.local.postgres {
        PostgresState::Primary { .. }
            if world.local.observation.waiting_for_fresh_pg_after_demote() =>
        {
            None
        }
        PostgresState::Primary { .. } => {
            Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast))
        }
        PostgresState::Offline => match &world.local.data_dir {
            DataDirState::Initialized(_) => Some(ReconcileAction::StartDetachedStandby),
            DataDirState::Missing => None,
        },
        PostgresState::Replica { .. } => None,
    }
}

fn leadership_held_by_self(world: &WorldView) -> bool {
    matches!(
        world.global.coordination.leadership,
        LeadershipView::HeldBySelf(_)
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        dcs::DcsTrust,
        state::{MemberId, UnixMillis},
    };

    use super::*;
    use crate::ha::types::{
        ApiVisibility, AuthorityProjection, CoordinationState, GlobalKnowledge, IneligibleReason,
        LeadershipView, LeaseEpoch, LocalKnowledge, NoPrimaryFence, NoPrimaryProjection,
        ObservationState, PeerKnowledge, PrimaryObservation, PublicationState, ShutdownMode,
        StorageState, SwitchoverState, WalPosition,
    };

    fn world(local: LocalKnowledge) -> WorldView {
        WorldView {
            local,
            global: GlobalKnowledge {
                coordination: CoordinationState {
                    trust: DcsTrust::FullQuorum,
                    leadership: LeadershipView::Open,
                    primary: PrimaryObservation::Absent,
                },
                switchover: SwitchoverState::None,
                peers: BTreeMap::new(),
                self_peer: PeerKnowledge {
                    eligibility: super::super::types::ElectionEligibility::Ineligible(
                        IneligibleReason::StartingUp,
                    ),
                    api: ApiVisibility::Unreachable,
                },
            },
        }
    }

    #[test]
    fn degraded_failsafe_keeps_stale_lease_instead_of_releasing_it() {
        let publication = PublicationGoal::Publish(AuthorityProjection::NoPrimary(
            NoPrimaryProjection::DcsDegraded {
                fence: NoPrimaryFence::None,
            },
        ));
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Offline,
            process: ProcessState::Idle,
            storage: StorageState::Healthy,
            required_roles_ready: false,
            publication: PublicationState::unknown(),
            observation: ObservationState {
                pg_observed_at: UnixMillis(100),
                last_start_success_at: None,
                last_promote_success_at: None,
                last_demote_success_at: None,
            },
        });
        let desired = DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
            publication: publication.clone(),
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![ReconcileAction::Publish(publication)]
        );
    }

    #[test]
    fn demoting_for_switchover_releases_lease_once_postgres_is_offline() {
        let world = WorldView {
            local: LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Offline,
                process: ProcessState::Idle,
                storage: StorageState::Healthy,
                required_roles_ready: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(100),
                    last_start_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                },
            },
            global: GlobalKnowledge {
                coordination: CoordinationState {
                    trust: DcsTrust::FullQuorum,
                    leadership: LeadershipView::HeldBySelf(LeaseEpoch {
                        holder: MemberId("node-a".to_string()),
                        generation: 5,
                    }),
                    primary: PrimaryObservation::Absent,
                },
                switchover: SwitchoverState::None,
                peers: BTreeMap::new(),
                self_peer: PeerKnowledge {
                    eligibility: super::super::types::ElectionEligibility::Ineligible(
                        IneligibleReason::StartingUp,
                    ),
                    api: ApiVisibility::Unreachable,
                },
            },
        };

        let desired = DesiredState {
            role: TargetRole::DemotingForSwitchover(MemberId("node-b".to_string())),
            publication: PublicationGoal::KeepCurrent,
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![ReconcileAction::ReleaseLease]
        );
    }

    #[test]
    fn matching_no_primary_projection_does_not_republish() {
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Offline,
            process: ProcessState::Idle,
            storage: StorageState::Healthy,
            required_roles_ready: false,
            publication: PublicationState::Projected(AuthorityProjection::NoPrimary(
                NoPrimaryProjection::LeaseOpen,
            )),
            observation: ObservationState {
                pg_observed_at: UnixMillis(100),
                last_start_success_at: None,
                last_promote_success_at: None,
                last_demote_success_at: None,
            },
        });
        let desired = DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: PublicationGoal::Publish(AuthorityProjection::NoPrimary(
                NoPrimaryProjection::LeaseOpen,
            )),
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![ReconcileAction::StartDetachedStandby]
        );
    }

    #[test]
    fn idle_missing_data_dir_does_not_start_detached_standby() {
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Missing,
            postgres: PostgresState::Offline,
            process: ProcessState::Idle,
            storage: StorageState::Healthy,
            required_roles_ready: false,
            publication: PublicationState::unknown(),
            observation: ObservationState {
                pg_observed_at: UnixMillis(100),
                last_start_success_at: None,
                last_promote_success_at: None,
                last_demote_success_at: None,
            },
        });
        let desired = DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: PublicationGoal::KeepCurrent,
            clear_switchover: false,
        };

        assert!(reconcile(&world, &desired).is_empty());
    }

    #[test]
    fn follower_replica_without_upstream_is_restarted_to_follow_authoritative_leader() {
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Replica {
                upstream: None,
                replication: super::super::types::ReplicationState::CatchingUp(WalPosition {
                    timeline: 1,
                    lsn: 42,
                }),
            },
            process: ProcessState::Idle,
            storage: StorageState::Healthy,
            required_roles_ready: false,
            publication: PublicationState::unknown(),
            observation: ObservationState {
                pg_observed_at: UnixMillis(100),
                last_start_success_at: None,
                last_promote_success_at: None,
                last_demote_success_at: None,
            },
        });
        let desired = DesiredState {
            role: TargetRole::Follower(FollowGoal {
                leader: MemberId("node-b".to_string()),
                recovery: RecoveryPlan::StartStreaming,
            }),
            publication: PublicationGoal::KeepCurrent,
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![ReconcileAction::Demote(ShutdownMode::Fast)]
        );
    }

    #[test]
    fn follower_replica_without_upstream_is_restarted_even_when_receiver_is_healthy() {
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Replica {
                upstream: None,
                replication: super::super::types::ReplicationState::Streaming(WalPosition {
                    timeline: 1,
                    lsn: 84,
                }),
            },
            process: ProcessState::Idle,
            storage: StorageState::Healthy,
            required_roles_ready: false,
            publication: PublicationState::unknown(),
            observation: ObservationState {
                pg_observed_at: UnixMillis(100),
                last_start_success_at: None,
                last_promote_success_at: None,
                last_demote_success_at: None,
            },
        });
        let desired = DesiredState {
            role: TargetRole::Follower(FollowGoal {
                leader: MemberId("node-b".to_string()),
                recovery: RecoveryPlan::StartStreaming,
            }),
            publication: PublicationGoal::KeepCurrent,
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![ReconcileAction::Demote(ShutdownMode::Fast)]
        );
    }
}
