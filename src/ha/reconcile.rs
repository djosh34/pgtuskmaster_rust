use super::types::{
    CoordinationAction, DataDirState, DesiredState, FailSafeGoal, FenceReason, FollowGoal,
    IdleReason, LeadershipView, LocalAction, LocalDataState, PostgresState, ProcessState,
    PublicationAction, PublicationGoal, PublicationState, ReconcilePlan, RecoveryPlan, TargetRole,
    WorldView,
};
use crate::process::jobs::{
    PostgresStartIntent, ProcessIntent, ReplicaProvisionIntent, ShutdownMode,
};

pub(crate) fn reconcile(world: &WorldView, desired: &DesiredState) -> ReconcilePlan {
    let publication_plan = reconcile_publication(&world.local.publication, desired);
    let switchover_plan = reconcile_switchover(world, desired);
    let role_plan = match &world.local.process {
        ProcessState::Running(_) => ReconcilePlan::default(),
        ProcessState::Idle | ProcessState::Failed(_) => reconcile_role(world, &desired.role),
    };

    publication_plan.merge(switchover_plan).merge(role_plan)
}

fn reconcile_publication(current: &PublicationState, desired: &DesiredState) -> ReconcilePlan {
    match &desired.publication {
        PublicationGoal::KeepCurrent => ReconcilePlan::default(),
        PublicationGoal::Publish(projection)
            if current == &PublicationState::Projected(projection.clone()) =>
        {
            ReconcilePlan::default()
        }
        publication => ReconcilePlan::publication(PublicationAction::Publish(publication.clone())),
    }
}

fn reconcile_switchover(world: &WorldView, desired: &DesiredState) -> ReconcilePlan {
    match (&world.global.switchover, desired.clear_switchover) {
        (super::types::SwitchoverState::Requested(_), true) => {
            ReconcilePlan::coordination(CoordinationAction::ClearSwitchover)
        }
        (super::types::SwitchoverState::None, _) | (_, false) => ReconcilePlan::default(),
    }
}

fn reconcile_role(world: &WorldView, target: &TargetRole) -> ReconcilePlan {
    match target {
        TargetRole::Leader(_) => match (&world.local.data_dir, &world.local.postgres) {
            (DataDirState::Missing, _) => ReconcilePlan::process(ProcessIntent::Bootstrap),
            (DataDirState::Initialized(LocalDataState::BootstrapEmpty), _) => {
                ReconcilePlan::process(ProcessIntent::Bootstrap)
            }
            (_, _) if world.local.observation.waiting_for_fresh_pg_after_start() => {
                ReconcilePlan::default()
            }
            (_, _) if world.local.observation.waiting_for_fresh_pg_after_promote() => {
                ReconcilePlan::default()
            }
            (DataDirState::Initialized(_), PostgresState::Offline) => {
                ReconcilePlan::process(ProcessIntent::Start(PostgresStartIntent::Primary))
            }
            (DataDirState::Initialized(_), PostgresState::Replica { .. }) => {
                ReconcilePlan::process(ProcessIntent::Promote)
            }
            (DataDirState::Initialized(_), PostgresState::Primary { .. }) => {
                if world.local.managed_roles_reconciled {
                    ReconcilePlan::default()
                } else {
                    ReconcilePlan::local(LocalAction::ReconcileManagedRoles)
                }
            }
        },
        TargetRole::Candidate(kind) => {
            ReconcilePlan::coordination(CoordinationAction::AcquireLease(kind.clone()))
        }
        TargetRole::Follower(goal) => reconcile_follow_role(world, goal),
        TargetRole::FailSafe(goal) => reconcile_failsafe_role(world, goal),
        TargetRole::DemotingForSwitchover(_) => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                ReconcilePlan::default()
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast))
            }
            PostgresState::Offline => {
                if leadership_held_by_self(world) {
                    ReconcilePlan::coordination(CoordinationAction::ReleaseLease)
                } else {
                    ReconcilePlan::default()
                }
            }
        },
        TargetRole::Fenced(reason) => reconcile_fenced_role(world, reason),
        TargetRole::Idle(reason) => reconcile_idle_role(world, reason),
    }
}

fn reconcile_follow_role(world: &WorldView, goal: &FollowGoal) -> ReconcilePlan {
    match goal.recovery {
        RecoveryPlan::None => ReconcilePlan::default(),
        RecoveryPlan::Basebackup => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                ReconcilePlan::default()
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast))
            }
            PostgresState::Offline => ReconcilePlan::process(ProcessIntent::ProvisionReplica(
                ReplicaProvisionIntent::BaseBackup {
                    leader: goal.leader.clone(),
                },
            )),
        },
        RecoveryPlan::Rewind => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                ReconcilePlan::default()
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast))
            }
            PostgresState::Offline => ReconcilePlan::process(ProcessIntent::ProvisionReplica(
                ReplicaProvisionIntent::PgRewind {
                    leader: goal.leader.clone(),
                },
            )),
        },
        RecoveryPlan::StartStreaming => {
            if world.local.observation.waiting_for_fresh_pg_after_start() {
                return ReconcilePlan::default();
            }
            if world.local.observation.waiting_for_fresh_pg_after_demote() {
                return ReconcilePlan::default();
            }

            match &world.local.postgres {
                PostgresState::Offline => {
                    ReconcilePlan::process(ProcessIntent::Start(PostgresStartIntent::Replica {
                        leader: goal.leader.clone(),
                    }))
                }
                PostgresState::Primary { .. } => {
                    ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast))
                }
                PostgresState::Replica {
                    upstream,
                    replication: _,
                } => match upstream {
                    Some(current_upstream) if current_upstream == &goal.leader => {
                        ReconcilePlan::default()
                    }
                    Some(_) => ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast)),
                    None => ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast)),
                },
            }
        }
    }
}

fn reconcile_failsafe_role(world: &WorldView, goal: &FailSafeGoal) -> ReconcilePlan {
    match goal {
        FailSafeGoal::PrimaryMustStop(_) => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                ReconcilePlan::default()
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Immediate))
            }
            PostgresState::Offline => ReconcilePlan::default(),
        },
        FailSafeGoal::ReplicaKeepFollowing(_) => ReconcilePlan::default(),
        FailSafeGoal::WaitForQuorum => match &world.local.postgres {
            PostgresState::Primary { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                ReconcilePlan::default()
            }
            PostgresState::Primary { .. } => {
                ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Immediate))
            }
            PostgresState::Replica { .. } => ReconcilePlan::default(),
            PostgresState::Offline => ReconcilePlan::default(),
        },
    }
}

fn reconcile_fenced_role(world: &WorldView, reason: &FenceReason) -> ReconcilePlan {
    match reason {
        FenceReason::StorageStalled if leadership_held_by_self(world) => {
            ReconcilePlan::coordination(CoordinationAction::ReleaseLease)
        }
        FenceReason::ForeignLeaderDetected | FenceReason::StorageStalled => {
            match &world.local.postgres {
                PostgresState::Primary { .. } | PostgresState::Replica { .. }
                    if world.local.observation.waiting_for_fresh_pg_after_demote() =>
                {
                    ReconcilePlan::default()
                }
                PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                    ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Immediate))
                }
                PostgresState::Offline => {
                    if leadership_held_by_self(world) {
                        ReconcilePlan::coordination(CoordinationAction::ReleaseLease)
                    } else {
                        ReconcilePlan::default()
                    }
                }
            }
        }
    }
}

fn reconcile_idle_role(world: &WorldView, _reason: &IdleReason) -> ReconcilePlan {
    match &world.local.postgres {
        PostgresState::Primary { .. }
            if world.local.observation.waiting_for_fresh_pg_after_demote() =>
        {
            ReconcilePlan::default()
        }
        PostgresState::Primary { .. } => {
            ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast))
        }
        PostgresState::Offline => match &world.local.data_dir {
            DataDirState::Initialized(_) => {
                ReconcilePlan::process(ProcessIntent::Start(PostgresStartIntent::DetachedStandby))
            }
            DataDirState::Missing => ReconcilePlan::default(),
        },
        PostgresState::Replica { .. } => ReconcilePlan::default(),
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
        process::jobs::ShutdownMode,
        state::{MemberId, UnixMillis},
    };

    use super::*;
    use crate::ha::types::{
        ApiVisibility, AuthorityProjection, CoordinationState, GlobalKnowledge, IneligibleReason,
        LeadershipView, LeaseEpoch, LocalKnowledge, NoPrimaryFence, NoPrimaryProjection,
        ObservationState, PeerKnowledge, PrimaryObservation, PublicationState, StorageState,
        SwitchoverState, WalPosition,
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
            managed_roles_reconciled: false,
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
            ReconcilePlan::publication(PublicationAction::Publish(publication))
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
                managed_roles_reconciled: false,
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
            ReconcilePlan::coordination(CoordinationAction::ReleaseLease)
        );
    }

    #[test]
    fn matching_no_primary_projection_does_not_republish() {
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Offline,
            process: ProcessState::Idle,
            storage: StorageState::Healthy,
            managed_roles_reconciled: false,
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
            ReconcilePlan::process(ProcessIntent::Start(PostgresStartIntent::DetachedStandby))
        );
    }

    #[test]
    fn idle_missing_data_dir_does_not_start_detached_standby() {
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Missing,
            postgres: PostgresState::Offline,
            process: ProcessState::Idle,
            storage: StorageState::Healthy,
            managed_roles_reconciled: false,
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
            managed_roles_reconciled: false,
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
            ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast))
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
            managed_roles_reconciled: false,
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
            ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast))
        );
    }
}
