use super::types::{
    AuthorityView, DataDirState, DesiredState, FailSafeGoal, FenceReason, FollowGoal, IdleReason,
    LeaseState, LocalDataState, PostgresState, ProcessState, PublicationGoal, PublicationState,
    ReconcileAction, RecoveryPlan, TargetRole, WorldView,
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
    let publish_action = match (
        &current.authority,
        &current.fence_cutoff,
        &desired.publication,
    ) {
        (_, _, PublicationGoal::KeepCurrent) => None,
        (
            AuthorityView::Primary {
                member: current_member,
                epoch: current_epoch,
            },
            current_cutoff,
            PublicationGoal::PublishPrimary { primary, epoch },
        ) if current_member == primary && current_epoch == epoch && current_cutoff.is_none() => {
            None
        }
        (
            AuthorityView::NoPrimary(current_reason),
            current_cutoff,
            PublicationGoal::PublishNoPrimary {
                reason,
                fence_cutoff,
            },
        ) if current_reason == reason && current_cutoff == fence_cutoff => None,
        (_, _, publication) => Some(ReconcileAction::Publish(publication.clone())),
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
            (DataDirState::Initialized(_), PostgresState::Primary { .. }) => None,
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
            PostgresState::Offline => match &world.global.lease {
                LeaseState::HeldByMe(_) => Some(ReconcileAction::ReleaseLease),
                LeaseState::HeldByPeer(_) | LeaseState::Unheld => None,
            },
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
                PostgresState::Replica { upstream, .. } => match upstream {
                    Some(current_upstream) if current_upstream == &goal.leader => None,
                    Some(_) => Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast)),
                    None => None,
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
        FenceReason::StorageStalled if matches!(world.global.lease, LeaseState::HeldByMe(_)) => {
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
                PostgresState::Offline => match &world.global.lease {
                    LeaseState::HeldByMe(_) => Some(ReconcileAction::ReleaseLease),
                    LeaseState::HeldByPeer(_) | LeaseState::Unheld => None,
                },
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
        PostgresState::Offline | PostgresState::Replica { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        dcs::state::DcsTrust,
        state::{MemberId, UnixMillis},
    };

    use super::*;
    use crate::ha::types::{
        ApiVisibility, AuthorityView, ElectionEligibility, GlobalKnowledge, IneligibleReason,
        LeaseEpoch, LocalKnowledge, ObservationState, PeerKnowledge, PublicationState,
        SwitchoverState,
    };

    #[test]
    fn degraded_failsafe_keeps_stale_lease_instead_of_releasing_it() {
        let publication = PublicationGoal::PublishNoPrimary {
            reason: super::super::types::NoPrimaryReason::DcsDegraded,
            fence_cutoff: None,
        };
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Offline,
            process: ProcessState::Idle,
            storage: super::super::types::StorageState::Healthy,
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
    fn storage_stalled_leader_releases_lease_before_demoting() {
        let publication = PublicationGoal::PublishNoPrimary {
            reason: super::super::types::NoPrimaryReason::Recovering,
            fence_cutoff: Some(super::super::types::FenceCutoff {
                epoch: LeaseEpoch {
                    holder: MemberId("node-a".to_string()),
                    generation: 7,
                },
                committed_lsn: 42,
            }),
        };
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Primary { committed_lsn: 42 },
            process: ProcessState::Idle,
            storage: super::super::types::StorageState::Stalled,
            publication: PublicationState::unknown(),
            observation: ObservationState {
                pg_observed_at: UnixMillis(100),
                last_start_success_at: None,
                last_promote_success_at: None,
                last_demote_success_at: None,
            },
        });
        let desired = DesiredState {
            role: TargetRole::Fenced(FenceReason::StorageStalled),
            publication: publication.clone(),
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![
                ReconcileAction::Publish(publication),
                ReconcileAction::ReleaseLease,
            ]
        );
    }

    #[test]
    fn follower_waits_for_new_pg_observation_after_demote_succeeds() {
        let publication = PublicationGoal::PublishPrimary {
            primary: MemberId("node-b".to_string()),
            epoch: LeaseEpoch {
                holder: MemberId("node-b".to_string()),
                generation: 7,
            },
        };
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Primary { committed_lsn: 42 },
            process: ProcessState::Idle,
            storage: super::super::types::StorageState::Healthy,
            publication: PublicationState {
                authority: AuthorityView::Unknown,
                fence_cutoff: None,
            },
            observation: ObservationState {
                pg_observed_at: UnixMillis(100),
                last_start_success_at: None,
                last_promote_success_at: None,
                last_demote_success_at: Some(UnixMillis(100)),
            },
        });
        let desired = DesiredState {
            role: TargetRole::Follower(FollowGoal {
                leader: MemberId("node-b".to_string()),
                recovery: RecoveryPlan::StartStreaming,
            }),
            publication: publication.clone(),
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![ReconcileAction::Publish(publication)]
        );
    }

    #[test]
    fn follower_does_not_demote_healthy_replica_when_upstream_is_unreported() {
        let publication = PublicationGoal::PublishPrimary {
            primary: MemberId("node-b".to_string()),
            epoch: LeaseEpoch {
                holder: MemberId("node-b".to_string()),
                generation: 7,
            },
        };
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Replica {
                upstream: None,
                replication: super::super::types::ReplicationState::Streaming(
                    super::super::types::WalPosition {
                        timeline: 1,
                        lsn: 42,
                    },
                ),
            },
            process: ProcessState::Idle,
            storage: super::super::types::StorageState::Healthy,
            publication: PublicationState::unknown(),
            observation: ObservationState {
                pg_observed_at: UnixMillis(100),
                last_start_success_at: Some(UnixMillis(1)),
                last_promote_success_at: None,
                last_demote_success_at: None,
            },
        });
        let desired = DesiredState {
            role: TargetRole::Follower(FollowGoal {
                leader: MemberId("node-b".to_string()),
                recovery: RecoveryPlan::StartStreaming,
            }),
            publication: publication.clone(),
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![ReconcileAction::Publish(publication)]
        );
    }

    #[test]
    fn leader_waits_for_fresh_observation_after_promote_succeeds() {
        let publication = PublicationGoal::PublishPrimary {
            primary: MemberId("node-a".to_string()),
            epoch: LeaseEpoch {
                holder: MemberId("node-a".to_string()),
                generation: 7,
            },
        };
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
            postgres: PostgresState::Replica {
                upstream: None,
                replication: super::super::types::ReplicationState::Streaming(
                    super::super::types::WalPosition {
                        timeline: 1,
                        lsn: 42,
                    },
                ),
            },
            process: ProcessState::Idle,
            storage: super::super::types::StorageState::Healthy,
            publication: PublicationState::unknown(),
            observation: ObservationState {
                pg_observed_at: UnixMillis(100),
                last_start_success_at: None,
                last_promote_success_at: Some(UnixMillis(100)),
                last_demote_success_at: None,
            },
        });
        let desired = DesiredState {
            role: TargetRole::Leader(LeaseEpoch {
                holder: MemberId("node-a".to_string()),
                generation: 7,
            }),
            publication: publication.clone(),
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![ReconcileAction::Publish(publication)]
        );
    }

    #[test]
    fn follower_demotes_before_running_pg_rewind() {
        let publication = PublicationGoal::PublishPrimary {
            primary: MemberId("node-b".to_string()),
            epoch: LeaseEpoch {
                holder: MemberId("node-b".to_string()),
                generation: 7,
            },
        };
        let world = world(LocalKnowledge {
            data_dir: DataDirState::Initialized(LocalDataState::Diverged(
                super::super::types::DivergenceState::RewindPossible,
            )),
            postgres: PostgresState::Replica {
                upstream: Some(MemberId("node-a".to_string())),
                replication: super::super::types::ReplicationState::Streaming(
                    super::super::types::WalPosition {
                        timeline: 1,
                        lsn: 42,
                    },
                ),
            },
            process: ProcessState::Idle,
            storage: super::super::types::StorageState::Healthy,
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
                recovery: RecoveryPlan::Rewind,
            }),
            publication: publication.clone(),
            clear_switchover: false,
        };

        assert_eq!(
            reconcile(&world, &desired),
            vec![
                ReconcileAction::Publish(publication),
                ReconcileAction::Demote(super::super::types::ShutdownMode::Fast),
            ]
        );
    }

    fn world(local: LocalKnowledge) -> WorldView {
        WorldView {
            local,
            global: GlobalKnowledge {
                dcs_trust: DcsTrust::NotTrusted,
                lease: LeaseState::HeldByMe(LeaseEpoch {
                    holder: MemberId("node-a".to_string()),
                    generation: 3,
                }),
                observed_lease: Some(LeaseEpoch {
                    holder: MemberId("node-a".to_string()),
                    generation: 3,
                }),
                observed_primary: Some(MemberId("node-a".to_string())),
                switchover: SwitchoverState::None,
                peers: BTreeMap::new(),
                self_peer: PeerKnowledge {
                    election: ElectionEligibility::Ineligible(IneligibleReason::NotReady),
                    api: ApiVisibility::Reachable,
                },
            },
        }
    }
}
