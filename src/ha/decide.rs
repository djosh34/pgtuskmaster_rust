use std::cmp::Ordering;

use crate::state::MemberId;

use super::types::{
    ApiVisibility, AuthorityProjection, Candidacy, DesiredState, ElectionEligibility, FailSafeGoal,
    FailureRecovery, FenceCutoff, FenceReason, FollowGoal, IdleReason, LeadershipView,
    LocalDataState, NoPrimaryFence, NoPrimaryProjection, PeerKnowledge, PeerLeaderState,
    PostgresState, ProcessAssessment, PublicationGoal, PublicationState, QuorumCoordinationState,
    RecoveryPlan, StorageState, SwitchoverState, TargetRole, WorldView,
};
use crate::state::LeaseEpoch;

pub(crate) fn decide(world: &WorldView, self_id: &MemberId) -> DesiredState {
    let Some(coordination) = world.global.coordination.as_quorum() else {
        return decide_no_quorum(world);
    };

    if world.local.storage == StorageState::Stalled {
        if let PostgresState::Primary { committed_lsn } = &world.local.postgres {
            let fence = active_epoch(coordination).map(|epoch| FenceCutoff {
                epoch,
                committed_lsn: *committed_lsn,
            });
            return DesiredState {
                role: TargetRole::Fenced(FenceReason::StorageStalled),
                publication: no_primary_publication(NoPrimaryProjection::Recovering {
                    epoch: active_epoch(coordination),
                    fence: fence
                        .map(NoPrimaryFence::Cutoff)
                        .unwrap_or(NoPrimaryFence::None),
                }),
                clear_switchover: false,
            };
        }
    }

    match &coordination.leadership {
        LeadershipView::HeldBySelf(epoch) => decide_as_lease_holder(world, self_id, epoch.clone()),
        LeadershipView::HeldByPeer { epoch, state } => {
            decide_under_foreign_leadership(world, epoch.clone(), state)
        }
        LeadershipView::Open => decide_without_lease(world, coordination, self_id),
    }
}

fn decide_no_quorum(world: &WorldView) -> DesiredState {
    let no_quorum_projection = || {
        no_primary_publication(NoPrimaryProjection::NoQuorum {
            fence: no_quorum_fence(world),
        })
    };

    match &world.local.postgres {
        PostgresState::Primary { committed_lsn } => {
            if let Some(epoch) = publication_epoch(&world.local.publication) {
                let cutoff = FenceCutoff {
                    epoch,
                    committed_lsn: *committed_lsn,
                };
                return DesiredState {
                    role: TargetRole::FailSafe(FailSafeGoal::PrimaryMustStop(cutoff.clone())),
                    publication: no_primary_publication(NoPrimaryProjection::NoQuorum {
                        fence: NoPrimaryFence::Cutoff(cutoff),
                    }),
                    clear_switchover: false,
                };
            }

            DesiredState {
                role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
                publication: no_quorum_projection(),
                clear_switchover: false,
            }
        }
        PostgresState::Replica { upstream, .. } => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::ReplicaKeepFollowing(upstream.clone())),
            publication: no_quorum_projection(),
            clear_switchover: false,
        },
        PostgresState::Offline => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
            publication: no_quorum_projection(),
            clear_switchover: false,
        },
    }
}

fn decide_under_foreign_leadership(
    world: &WorldView,
    epoch: LeaseEpoch,
    state: &PeerLeaderState,
) -> DesiredState {
    let publication = match state {
        PeerLeaderState::PrimaryReady => primary_publication(epoch.clone()),
        PeerLeaderState::Recovering | PeerLeaderState::Unreachable => {
            no_primary_publication(NoPrimaryProjection::Recovering {
                epoch: Some(epoch.clone()),
                fence: NoPrimaryFence::None,
            })
        }
    };

    match (&world.local.postgres, state) {
        (PostgresState::Primary { .. }, _) => DesiredState {
            role: TargetRole::Fenced(FenceReason::ForeignLeaderDetected),
            publication,
            clear_switchover: false,
        },
        (PostgresState::Offline | PostgresState::Replica { .. }, PeerLeaderState::PrimaryReady) => {
            DesiredState {
                role: TargetRole::Follower(follow_goal(world, epoch.holder)),
                publication,
                clear_switchover: false,
            }
        }
        (PostgresState::Offline | PostgresState::Replica { .. }, _) => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication,
            clear_switchover: false,
        },
    }
}

fn decide_as_lease_holder(
    world: &WorldView,
    self_id: &MemberId,
    epoch: LeaseEpoch,
) -> DesiredState {
    let publication = leader_publication(world, self_id, &epoch);
    let allow_self_switchover_target = false;
    let Some(coordination) = world.global.coordination.as_quorum() else {
        return decide_no_quorum(world);
    };

    match resolve_switchover(
        coordination,
        &world.global.self_peer,
        self_id,
        allow_self_switchover_target,
    ) {
        ResolvedSwitchover::NotRequested => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication,
            clear_switchover: false,
        },
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication,
            clear_switchover: true,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::DemotingForSwitchover(target),
            publication: PublicationGoal::KeepCurrent,
            clear_switchover: false,
        },
        ResolvedSwitchover::Pending => DesiredState {
            role: TargetRole::Leader(epoch),
            publication,
            clear_switchover: false,
        },
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Leader(epoch),
            publication,
            clear_switchover: true,
        },
    }
}

fn decide_without_lease(
    world: &WorldView,
    coordination: &QuorumCoordinationState,
    self_id: &MemberId,
) -> DesiredState {
    match resolve_switchover(coordination, &world.global.self_peer, self_id, true) {
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Candidate(Candidacy::TargetedSwitchover(target)),
            publication: lease_open_publication(),
            clear_switchover: false,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingTarget(target)),
            publication: lease_open_publication(),
            clear_switchover: false,
        },
        ResolvedSwitchover::Pending => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: lease_open_publication(),
            clear_switchover: false,
        },
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: lease_open_publication(),
            clear_switchover: true,
        },
        ResolvedSwitchover::NotRequested
            if best_failover_candidate(&coordination.peers, &world.global.self_peer, self_id)
                == Some(self_id.clone()) =>
        {
            DesiredState {
                role: TargetRole::Candidate(candidacy_kind(world)),
                publication: lease_open_publication(),
                clear_switchover: false,
            }
        }
        ResolvedSwitchover::NotRequested => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: lease_open_publication(),
            clear_switchover: false,
        },
    }
}

fn leader_publication(
    world: &WorldView,
    self_id: &MemberId,
    epoch: &LeaseEpoch,
) -> PublicationGoal {
    match &world.local.postgres {
        PostgresState::Primary { .. } => primary_publication(epoch.clone()),
        PostgresState::Offline | PostgresState::Replica { .. } => {
            no_primary_publication(NoPrimaryProjection::Recovering {
                epoch: Some(LeaseEpoch {
                    holder: self_id.clone(),
                    generation: epoch.generation,
                }),
                fence: NoPrimaryFence::None,
            })
        }
    }
}

fn follow_goal(world: &WorldView, leader: MemberId) -> FollowGoal {
    let recovery = match &world.local.data_dir {
        super::types::DataDirState::Missing => RecoveryPlan::Basebackup,
        super::types::DataDirState::Initialized(LocalDataState::BootstrapEmpty) => {
            RecoveryPlan::Basebackup
        }
        super::types::DataDirState::Initialized(LocalDataState::ConsistentReplica) => {
            match &world.local.postgres {
                PostgresState::Replica { upstream, .. } if upstream.as_ref() == Some(&leader) => {
                    RecoveryPlan::None
                }
                PostgresState::Replica { .. }
                | PostgresState::Offline
                | PostgresState::Primary { .. } => {
                    if rewind_failed_and_requires_basebackup(&world.local.process) {
                        RecoveryPlan::Basebackup
                    } else {
                        RecoveryPlan::StartStreaming
                    }
                }
            }
        }
        super::types::DataDirState::Initialized(LocalDataState::Diverged(state)) => match state {
            super::types::DivergenceState::RewindPossible => {
                if rewind_failed_and_requires_basebackup(&world.local.process) {
                    RecoveryPlan::Basebackup
                } else if world
                    .local
                    .observation
                    .basebackup_completed_awaiting_start()
                {
                    RecoveryPlan::StartStreaming
                } else {
                    RecoveryPlan::Rewind
                }
            }
            super::types::DivergenceState::BasebackupRequired => RecoveryPlan::Basebackup,
        },
    };

    FollowGoal { leader, recovery }
}

fn rewind_failed_and_requires_basebackup(process: &ProcessAssessment) -> bool {
    matches!(
        process,
        ProcessAssessment::Failed(super::types::JobFailure {
            job: crate::process::jobs::ActiveJobKind::PgRewind,
            recovery: FailureRecovery::FallbackToBasebackup,
        })
    )
}

fn candidacy_kind(world: &WorldView) -> Candidacy {
    match &world.local.data_dir {
        super::types::DataDirState::Missing
        | super::types::DataDirState::Initialized(LocalDataState::BootstrapEmpty) => {
            Candidacy::Bootstrap
        }
        _ => {
            if matches!(
                world.local.publication,
                PublicationState::Projected(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::NoQuorum { .. }
                ))
            ) {
                Candidacy::ResumeAfterOutage
            } else {
                Candidacy::Failover
            }
        }
    }
}

fn active_epoch(coordination: &QuorumCoordinationState) -> Option<LeaseEpoch> {
    match &coordination.leadership {
        LeadershipView::Open => None,
        LeadershipView::HeldBySelf(epoch) | LeadershipView::HeldByPeer { epoch, .. } => {
            Some(epoch.clone())
        }
    }
}

fn primary_publication(epoch: LeaseEpoch) -> PublicationGoal {
    PublicationGoal::Publish(AuthorityProjection::Primary(epoch))
}

fn no_primary_publication(projection: NoPrimaryProjection) -> PublicationGoal {
    PublicationGoal::Publish(AuthorityProjection::NoPrimary(projection))
}

fn lease_open_publication() -> PublicationGoal {
    no_primary_publication(NoPrimaryProjection::LeaseOpen)
}

fn publication_epoch(publication: &PublicationState) -> Option<LeaseEpoch> {
    match publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => Some(epoch.clone()),
        PublicationState::Projected(AuthorityProjection::NoPrimary(
            NoPrimaryProjection::Recovering {
                epoch: Some(epoch), ..
            },
        )) => Some(epoch.clone()),
        PublicationState::Unknown
        | PublicationState::Projected(AuthorityProjection::NoPrimary(
            NoPrimaryProjection::NoQuorum { .. }
            | NoPrimaryProjection::LeaseOpen
            | NoPrimaryProjection::Recovering { epoch: None, .. }
            | NoPrimaryProjection::SwitchoverRejected(_),
        )) => None,
    }
}

fn no_quorum_fence(world: &WorldView) -> NoPrimaryFence {
    match (
        &world.local.postgres,
        publication_epoch(&world.local.publication),
    ) {
        (PostgresState::Primary { committed_lsn }, Some(epoch)) => {
            NoPrimaryFence::Cutoff(FenceCutoff {
                epoch,
                committed_lsn: *committed_lsn,
            })
        }
        _ => NoPrimaryFence::None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolvedSwitchover {
    NotRequested,
    Proceed(MemberId),
    Pending,
    Abandon,
}

fn resolve_switchover(
    coordination: &QuorumCoordinationState,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
    allow_self_target: bool,
) -> ResolvedSwitchover {
    match &coordination.switchover {
        SwitchoverState::None => ResolvedSwitchover::NotRequested,
        SwitchoverState::AnyHealthyReplica => {
            best_switchover_target(&coordination.peers, self_peer, self_id, allow_self_target)
                .map_or(ResolvedSwitchover::Pending, ResolvedSwitchover::Proceed)
        }
        SwitchoverState::Specific(member_id) => {
            if member_id == self_id {
                if allow_self_target && switchover_target_is_valid(self_peer) {
                    ResolvedSwitchover::Proceed(member_id.clone())
                } else {
                    ResolvedSwitchover::Abandon
                }
            } else if coordination
                .peers
                .get(member_id)
                .is_some_and(switchover_target_is_valid)
            {
                ResolvedSwitchover::Proceed(member_id.clone())
            } else {
                ResolvedSwitchover::Abandon
            }
        }
    }
}

fn best_switchover_target(
    peers: &std::collections::BTreeMap<MemberId, PeerKnowledge>,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
    allow_self_target: bool,
) -> Option<MemberId> {
    if allow_self_target && switchover_target_is_valid(self_peer) {
        return Some(self_id.clone());
    }

    let peer_candidate = peers
        .iter()
        .filter(|(_, peer)| switchover_target_is_valid(peer))
        .map(|(member_id, peer)| (member_id.clone(), peer))
        .max_by(|(left_id, left_peer), (right_id, right_peer)| {
            compare_candidate_eligibility(
                left_id,
                &left_peer.eligibility,
                right_id,
                &right_peer.eligibility,
            )
        })
        .map(|(member_id, _)| member_id);

    peer_candidate
}

fn best_failover_candidate(
    peers: &std::collections::BTreeMap<MemberId, PeerKnowledge>,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
) -> Option<MemberId> {
    let peer_candidate = peers
        .iter()
        .filter(|(_, peer)| !matches!(peer.eligibility, ElectionEligibility::Ineligible(_)))
        .map(|(member_id, peer)| (member_id.clone(), peer))
        .max_by(|(left_id, left_peer), (right_id, right_peer)| {
            compare_candidate_eligibility(
                left_id,
                &left_peer.eligibility,
                right_id,
                &right_peer.eligibility,
            )
        })
        .map(|(member_id, _)| member_id);

    if matches!(self_peer.eligibility, ElectionEligibility::Ineligible(_)) {
        return peer_candidate;
    }

    match peer_candidate {
        Some(peer_id) => {
            let Some(peer) = peers.get(&peer_id) else {
                return Some(self_id.clone());
            };
            if compare_candidate_eligibility(
                self_id,
                &self_peer.eligibility,
                &peer_id,
                &peer.eligibility,
            ) == Ordering::Greater
            {
                Some(self_id.clone())
            } else {
                Some(peer_id)
            }
        }
        None => Some(self_id.clone()),
    }
}

fn switchover_target_is_valid(peer: &PeerKnowledge) -> bool {
    matches!(peer.api, ApiVisibility::Reachable)
        && matches!(peer.eligibility, ElectionEligibility::PromoteEligible(_))
}

fn compare_candidate_eligibility(
    left_id: &MemberId,
    left: &ElectionEligibility,
    right_id: &MemberId,
    right: &ElectionEligibility,
) -> Ordering {
    match (left, right) {
        (
            ElectionEligibility::PromoteEligible(left_pos),
            ElectionEligibility::PromoteEligible(right_pos),
        ) => left_pos.cmp(right_pos).then_with(|| right_id.cmp(left_id)),
        (ElectionEligibility::PromoteEligible(_), ElectionEligibility::BootstrapEligible) => {
            Ordering::Greater
        }
        (ElectionEligibility::BootstrapEligible, ElectionEligibility::PromoteEligible(_)) => {
            Ordering::Less
        }
        (ElectionEligibility::BootstrapEligible, ElectionEligibility::BootstrapEligible) => {
            right_id.cmp(left_id)
        }
        (
            ElectionEligibility::PromoteEligible(_) | ElectionEligibility::BootstrapEligible,
            ElectionEligibility::Ineligible(_),
        ) => Ordering::Greater,
        (
            ElectionEligibility::Ineligible(_),
            ElectionEligibility::PromoteEligible(_) | ElectionEligibility::BootstrapEligible,
        ) => Ordering::Less,
        (ElectionEligibility::Ineligible(_), ElectionEligibility::Ineligible(_)) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{best_failover_candidate, decide};
    use crate::{
        dcs::DcsQuorumState,
        state::{LeaseEpoch, MemberId, UnixMillis},
    };

    use super::super::types::{
        ApiVisibility, AuthorityProjection, Candidacy, CoordinationState, DataDirState,
        DesiredState, DivergenceState, ElectionEligibility, FailSafeGoal, FollowGoal,
        GlobalKnowledge, IdleReason, IneligibleReason, LeadershipView, LocalDataState,
        LocalKnowledge, NoPrimaryProjection, ObservationState, ObservedPrimary, PeerKnowledge,
        PeerLeaderState, PostgresState, PrimaryObservation, ProcessAssessment, PublicationGoal,
        PublicationState, QuorumCoordinationState, RecoveryPlan, ReplicationState, StorageState,
        SwitchoverState, TargetRole, WalPosition, WorldView,
    };

    fn promote_peer(lsn: u64) -> PeerKnowledge {
        PeerKnowledge {
            eligibility: ElectionEligibility::PromoteEligible(WalPosition { timeline: 1, lsn }),
            api: ApiVisibility::Reachable,
        }
    }

    fn bootstrap_peer() -> PeerKnowledge {
        PeerKnowledge {
            eligibility: ElectionEligibility::BootstrapEligible,
            api: ApiVisibility::Reachable,
        }
    }

    fn world(local: LocalKnowledge, self_peer: PeerKnowledge) -> WorldView {
        WorldView {
            local,
            global: GlobalKnowledge {
                coordination: CoordinationState::Quorum(Box::new(QuorumCoordinationState {
                    dcs: DcsQuorumState {
                        leadership: None,
                        switchover: SwitchoverState::None,
                        members: BTreeMap::new(),
                    },
                    leadership: LeadershipView::Open,
                    primary: PrimaryObservation::Absent,
                    switchover: SwitchoverState::None,
                    peers: BTreeMap::new(),
                })),
                self_peer,
            },
        }
    }

    #[test]
    fn best_failover_candidate_includes_self_in_ranking() {
        let self_id = MemberId("node-a".to_string());
        let peers = BTreeMap::from([(MemberId("node-b".to_string()), promote_peer(10))]);

        assert_eq!(
            best_failover_candidate(&peers, &promote_peer(20), &self_id),
            Some(self_id)
        );
    }

    #[test]
    fn best_failover_candidate_prefers_higher_ranked_peer() {
        let self_id = MemberId("node-a".to_string());
        let peer_id = MemberId("node-b".to_string());
        let peers = BTreeMap::from([(peer_id.clone(), promote_peer(20))]);

        assert_eq!(
            best_failover_candidate(&peers, &promote_peer(10), &self_id),
            Some(peer_id)
        );
    }

    #[test]
    fn best_failover_candidate_prefers_bootstrap_peer_over_ineligible_self() {
        let self_id = MemberId("node-a".to_string());
        let peer_id = MemberId("node-b".to_string());
        let peers = BTreeMap::from([(peer_id.clone(), bootstrap_peer())]);

        assert_eq!(
            best_failover_candidate(
                &peers,
                &PeerKnowledge {
                    eligibility: ElectionEligibility::Ineligible(IneligibleReason::NotReady),
                    api: ApiVisibility::Reachable,
                },
                &self_id,
            ),
            Some(peer_id)
        );
    }

    #[test]
    fn no_quorum_keeps_replica_in_failsafe() {
        let self_id = MemberId("node-a".to_string());
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Replica {
                    upstream: Some(MemberId("node-b".to_string())),
                    replication: ReplicationState::Streaming(WalPosition {
                        timeline: 1,
                        lsn: 42,
                    }),
                },
                process: ProcessAssessment::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::Projected(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::LeaseOpen,
                )),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                    last_local_timeline: None,
                    last_local_system_identifier: None,
                },
            },
            promote_peer(42),
        );
        world.global.coordination = CoordinationState::NoQuorum;

        assert_eq!(
            decide(&world, &self_id),
            DesiredState {
                role: TargetRole::FailSafe(FailSafeGoal::ReplicaKeepFollowing(Some(MemberId(
                    "node-b".to_string(),
                )))),
                publication: PublicationGoal::Publish(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::NoQuorum {
                        fence: super::super::types::NoPrimaryFence::None,
                    },
                )),
                clear_switchover: false,
            }
        );
    }

    #[test]
    fn sampled_primary_without_lease_promotes_best_candidate() {
        let self_id = MemberId("node-a".to_string());
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Offline,
                process: ProcessAssessment::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                    last_local_timeline: None,
                    last_local_system_identifier: None,
                },
            },
            promote_peer(42),
        );
        if let Some(coordination) = world.global.coordination.as_quorum_mut() {
            coordination.primary = PrimaryObservation::Observed(ObservedPrimary {
                member: MemberId("node-b".to_string()),
                timeline: None,
                system_identifier: None,
            });
        }

        assert_eq!(
            decide(&world, &self_id).role,
            TargetRole::Candidate(Candidacy::Failover)
        );
    }

    #[test]
    fn basebackup_success_on_diverged_data_transitions_to_start_streaming() {
        let self_id = MemberId("node-b".to_string());
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::Diverged(
                    DivergenceState::RewindPossible,
                )),
                postgres: PostgresState::Offline,
                process: ProcessAssessment::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(100),
                    last_start_success_at: Some(UnixMillis(10)),
                    last_basebackup_success_at: Some(UnixMillis(20)),
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                    last_local_timeline: None,
                    last_local_system_identifier: None,
                },
            },
            promote_peer(42),
        );
        if let Some(coordination) = world.global.coordination.as_quorum_mut() {
            coordination.leadership = LeadershipView::HeldByPeer {
                epoch: LeaseEpoch {
                    holder: MemberId("node-a".to_string()),
                    generation: 7,
                },
                state: PeerLeaderState::PrimaryReady,
            };
        }

        assert_eq!(
            decide(&world, &self_id).role,
            TargetRole::Follower(FollowGoal {
                leader: MemberId("node-a".to_string()),
                recovery: RecoveryPlan::StartStreaming,
            })
        );
    }

    #[test]
    fn idle_when_no_leader_no_candidate_and_no_switchover() {
        let self_id = MemberId("node-a".to_string());
        let world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Offline,
                process: ProcessAssessment::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                    last_local_timeline: None,
                    last_local_system_identifier: None,
                },
            },
            PeerKnowledge {
                eligibility: ElectionEligibility::Ineligible(IneligibleReason::StartingUp),
                api: ApiVisibility::Unreachable,
            },
        );

        assert_eq!(
            decide(&world, &self_id).role,
            TargetRole::Idle(IdleReason::AwaitingLeader)
        );
    }

    #[test]
    fn generic_switchover_request_waits_for_future_eligible_target() {
        let self_id = MemberId("node-a".to_string());
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Primary { committed_lsn: 42 },
                process: ProcessAssessment::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                    last_local_timeline: None,
                    last_local_system_identifier: None,
                },
            },
            promote_peer(42),
        );
        if let Some(coordination) = world.global.coordination.as_quorum_mut() {
            coordination.leadership = LeadershipView::HeldBySelf(LeaseEpoch {
                holder: self_id.clone(),
                generation: 7,
            });
            coordination.switchover = SwitchoverState::AnyHealthyReplica;
            coordination.peers = BTreeMap::from([(
                MemberId("node-b".to_string()),
                PeerKnowledge {
                    eligibility: ElectionEligibility::Ineligible(IneligibleReason::NotReady),
                    api: ApiVisibility::Reachable,
                },
            )]);
        }

        assert_eq!(
            decide(&world, &self_id),
            DesiredState {
                role: TargetRole::Leader(LeaseEpoch {
                    holder: self_id.clone(),
                    generation: 7,
                }),
                publication: PublicationGoal::Publish(AuthorityProjection::Primary(LeaseEpoch {
                    holder: MemberId("node-a".to_string()),
                    generation: 7,
                },)),
                clear_switchover: false,
            }
        );
    }

    #[test]
    fn lease_holder_replica_keeps_handoff_target_for_generic_switchover_after_winning_lease() {
        let self_id = MemberId("node-c".to_string());
        let epoch = LeaseEpoch {
            holder: self_id.clone(),
            generation: 7,
        };
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Replica {
                    upstream: None,
                    replication: ReplicationState::Streaming(WalPosition {
                        timeline: 1,
                        lsn: 50,
                    }),
                },
                process: ProcessAssessment::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                    last_local_timeline: None,
                    last_local_system_identifier: None,
                },
            },
            promote_peer(50),
        );
        if let Some(coordination) = world.global.coordination.as_quorum_mut() {
            coordination.leadership = LeadershipView::HeldBySelf(epoch.clone());
            coordination.switchover = SwitchoverState::AnyHealthyReplica;
            coordination.peers =
                BTreeMap::from([(MemberId("node-a".to_string()), promote_peer(40))]);
        }

        assert_eq!(
            decide(&world, &self_id),
            DesiredState {
                role: TargetRole::DemotingForSwitchover(MemberId("node-a".to_string())),
                publication: PublicationGoal::KeepCurrent,
                clear_switchover: false,
            }
        );
    }
}
