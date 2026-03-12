use std::cmp::Ordering;

use crate::{dcs::state::DcsTrust, state::MemberId};

use super::types::{
    ApiVisibility, Candidacy, DesiredState, ElectionEligibility, FailSafeGoal, FailureRecovery,
    FenceCutoff, FenceReason, FollowGoal, IdleReason, LeaseEpoch, LeaseState, LocalDataState,
    NoPrimaryReason, PeerKnowledge, PostgresState, ProcessState, PublicationGoal, RecoveryPlan,
    StorageState, SwitchoverState, SwitchoverTarget, TargetRole, WalPosition, WorldView,
};

pub(crate) fn decide(world: &WorldView, self_id: &MemberId) -> DesiredState {
    if !matches!(world.global.dcs_trust, DcsTrust::FullQuorum) {
        return decide_degraded(world);
    }

    if world.local.storage == StorageState::Stalled {
        if let (PostgresState::Primary { committed_lsn }, Some(epoch)) =
            (&world.local.postgres, active_or_observed_epoch(world))
        {
            let cutoff = FenceCutoff {
                epoch,
                committed_lsn: *committed_lsn,
            };
            return DesiredState {
                role: TargetRole::Fenced(FenceReason::StorageStalled),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::Recovering,
                    fence_cutoff: Some(cutoff),
                },
                clear_switchover: false,
            };
        }
    }

    match &world.global.lease {
        LeaseState::HeldByMe(epoch) => decide_as_lease_holder(world, self_id, epoch.clone()),
        LeaseState::HeldByPeer(epoch) => {
            let publication = PublicationGoal::PublishPrimary {
                primary: epoch.holder.clone(),
                epoch: epoch.clone(),
            };
            match &world.local.postgres {
                PostgresState::Primary { .. } => DesiredState {
                    role: TargetRole::Fenced(FenceReason::ForeignLeaderDetected),
                    publication,
                    clear_switchover: false,
                },
                PostgresState::Offline | PostgresState::Replica { .. } => DesiredState {
                    role: TargetRole::Follower(follow_goal(world, epoch.holder.clone())),
                    publication,
                    clear_switchover: false,
                },
            }
        }
        LeaseState::Unheld => {
            if let Some(epoch) = observed_foreign_lease(world, self_id) {
                let publication = PublicationGoal::PublishPrimary {
                    primary: epoch.holder.clone(),
                    epoch: epoch.clone(),
                };
                return match &world.local.postgres {
                    PostgresState::Primary { .. } => DesiredState {
                        role: TargetRole::Fenced(FenceReason::ForeignLeaderDetected),
                        publication,
                        clear_switchover: false,
                    },
                    PostgresState::Offline => DesiredState {
                        role: TargetRole::Idle(IdleReason::AwaitingLeader),
                        publication,
                        clear_switchover: false,
                    },
                    PostgresState::Replica { .. } => {
                        if world.global.observed_primary.as_ref() == Some(&epoch.holder) {
                            DesiredState {
                                role: TargetRole::Follower(follow_goal(
                                    world,
                                    epoch.holder.clone(),
                                )),
                                publication,
                                clear_switchover: false,
                            }
                        } else {
                            DesiredState {
                                role: TargetRole::Idle(IdleReason::AwaitingLeader),
                                publication,
                                clear_switchover: false,
                            }
                        }
                    }
                };
            }

            decide_without_lease(world, self_id)
        }
    }
}

fn decide_degraded(world: &WorldView) -> DesiredState {
    match &world.local.postgres {
        PostgresState::Primary { committed_lsn } => {
            if let Some(epoch) = active_or_observed_epoch(world) {
                let cutoff = FenceCutoff {
                    epoch,
                    committed_lsn: *committed_lsn,
                };
                return DesiredState {
                    role: TargetRole::FailSafe(FailSafeGoal::PrimaryMustStop(cutoff.clone())),
                    publication: PublicationGoal::PublishNoPrimary {
                        reason: NoPrimaryReason::DcsDegraded,
                        fence_cutoff: Some(cutoff),
                    },
                    clear_switchover: false,
                };
            }

            DesiredState {
                role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::DcsDegraded,
                    fence_cutoff: None,
                },
                clear_switchover: false,
            }
        }
        PostgresState::Replica { upstream, .. } => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::ReplicaKeepFollowing(upstream.clone())),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::DcsDegraded,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
        PostgresState::Offline => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::DcsDegraded,
                fence_cutoff: None,
            },
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

    match resolve_switchover(world, self_id, false) {
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
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Leader(epoch),
            publication,
            clear_switchover: true,
        },
    }
}

fn decide_without_lease(world: &WorldView, self_id: &MemberId) -> DesiredState {
    if let Some(leader) = world
        .global
        .observed_primary
        .clone()
        .filter(|leader| leader != self_id)
    {
        return DesiredState {
            role: TargetRole::Follower(follow_goal(world, leader)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: false,
        };
    }

    match resolve_switchover(world, self_id, true) {
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Candidate(Candidacy::TargetedSwitchover(target)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingTarget(target)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: true,
        },
        ResolvedSwitchover::NotRequested
            if best_failover_candidate(&world.global.peers, &world.global.self_peer, self_id)
                == Some(self_id.clone()) =>
        {
            DesiredState {
                role: TargetRole::Candidate(candidacy_kind(world)),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::LeaseOpen,
                    fence_cutoff: None,
                },
                clear_switchover: false,
            }
        }
        ResolvedSwitchover::NotRequested => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
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
        PostgresState::Primary { .. } => PublicationGoal::PublishPrimary {
            primary: self_id.clone(),
            epoch: epoch.clone(),
        },
        PostgresState::Offline | PostgresState::Replica { .. } => {
            PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::Recovering,
                fence_cutoff: None,
            }
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
                } else {
                    RecoveryPlan::Rewind
                }
            }
            super::types::DivergenceState::BasebackupRequired => RecoveryPlan::Basebackup,
        },
    };

    FollowGoal { leader, recovery }
}

fn rewind_failed_and_requires_basebackup(process: &ProcessState) -> bool {
    matches!(
        process,
        ProcessState::Failed(super::types::JobFailure {
            job: super::types::JobKind::PgRewind,
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
                world.local.publication.authority,
                super::types::AuthorityView::NoPrimary(NoPrimaryReason::DcsDegraded)
            ) {
                Candidacy::ResumeAfterOutage
            } else {
                Candidacy::Failover
            }
        }
    }
}

fn active_or_observed_epoch(world: &WorldView) -> Option<LeaseEpoch> {
    match &world.global.lease {
        LeaseState::HeldByMe(epoch) | LeaseState::HeldByPeer(epoch) => Some(epoch.clone()),
        LeaseState::Unheld => world.global.observed_lease.clone(),
    }
}

fn observed_foreign_lease(world: &WorldView, self_id: &MemberId) -> Option<LeaseEpoch> {
    match &world.global.lease {
        LeaseState::HeldByPeer(epoch) if &epoch.holder != self_id => Some(epoch.clone()),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolvedSwitchover {
    NotRequested,
    Proceed(MemberId),
    Abandon,
}

fn resolve_switchover(
    world: &WorldView,
    self_id: &MemberId,
    allow_self_target: bool,
) -> ResolvedSwitchover {
    match &world.global.switchover {
        SwitchoverState::None => ResolvedSwitchover::NotRequested,
        SwitchoverState::Requested(request) => match &request.target {
            SwitchoverTarget::AnyHealthyReplica => best_switchover_target(
                &world.global.peers,
                &world.global.self_peer,
                self_id,
                allow_self_target,
            )
            .map_or(ResolvedSwitchover::Abandon, ResolvedSwitchover::Proceed),
            SwitchoverTarget::Specific(member_id) => {
                if member_id == self_id {
                    if allow_self_target && switchover_target_is_valid(&world.global.self_peer) {
                        ResolvedSwitchover::Proceed(member_id.clone())
                    } else {
                        ResolvedSwitchover::Abandon
                    }
                } else if world
                    .global
                    .peers
                    .get(member_id)
                    .is_some_and(switchover_target_is_valid)
                {
                    ResolvedSwitchover::Proceed(member_id.clone())
                } else {
                    ResolvedSwitchover::Abandon
                }
            }
        },
    }
}

fn best_switchover_target(
    peers: &std::collections::BTreeMap<MemberId, PeerKnowledge>,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
    allow_self_target: bool,
) -> Option<MemberId> {
    let peer_candidate = peers
        .iter()
        .filter(|(_, peer)| switchover_target_is_valid(peer))
        .map(|(member_id, peer)| (member_id.clone(), peer))
        .max_by(|(left_id, left_peer), (right_id, right_peer)| {
            compare_switchover_candidates(left_id, left_peer, right_id, right_peer)
        })
        .map(|(member_id, _)| member_id);

    if allow_self_target && switchover_target_is_valid(self_peer) {
        let self_candidate = Some(self_id.clone());
        return match (peer_candidate, self_candidate) {
            (Some(peer_id), Some(self_id)) => {
                if compare_self_to_peer(self_peer, &self_id, peers.get(&peer_id), &peer_id)
                    == Ordering::Greater
                {
                    Some(self_id)
                } else {
                    Some(peer_id)
                }
            }
            (Some(peer_id), None) => Some(peer_id),
            (None, Some(self_id)) => Some(self_id),
            (None, None) => None,
        };
    }

    peer_candidate
}

fn best_failover_candidate(
    peers: &std::collections::BTreeMap<MemberId, PeerKnowledge>,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
) -> Option<MemberId> {
    let peer_candidate = peers
        .iter()
        .filter(|(_, peer)| classify_candidate(peer).is_some())
        .map(|(member_id, peer)| (member_id.clone(), peer))
        .max_by(|(left_id, left_peer), (right_id, right_peer)| {
            compare_candidate_rank(
                candidate_rank(&left_peer.election),
                left_id,
                candidate_rank(&right_peer.election),
                right_id,
            )
        })
        .map(|(member_id, _)| member_id);

    if classify_candidate(self_peer).is_none() {
        return peer_candidate;
    }

    match peer_candidate {
        Some(peer_id) => {
            let peer_rank = peers
                .get(&peer_id)
                .map(|peer| candidate_rank(&peer.election));
            if compare_candidate_rank(
                candidate_rank(&self_peer.election),
                self_id,
                peer_rank.flatten(),
                &peer_id,
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

fn compare_self_to_peer(
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
    peer: Option<&PeerKnowledge>,
    peer_id: &MemberId,
) -> Ordering {
    match peer {
        Some(peer) => compare_switchover_candidates(self_id, self_peer, peer_id, peer),
        None => Ordering::Greater,
    }
}

fn switchover_target_is_valid(peer: &PeerKnowledge) -> bool {
    matches!(peer.api, ApiVisibility::Reachable)
        && matches!(peer.election, ElectionEligibility::PromoteEligible(_))
}

fn compare_switchover_candidates(
    left_id: &MemberId,
    left_peer: &PeerKnowledge,
    right_id: &MemberId,
    right_peer: &PeerKnowledge,
) -> Ordering {
    compare_candidate_rank(
        candidate_rank(&left_peer.election),
        left_id,
        candidate_rank(&right_peer.election),
        right_id,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CandidateRank {
    Promote(WalPosition),
    Bootstrap,
}

fn candidate_rank(value: &ElectionEligibility) -> Option<CandidateRank> {
    match value {
        ElectionEligibility::PromoteEligible(position) => {
            Some(CandidateRank::Promote(position.clone()))
        }
        ElectionEligibility::BootstrapEligible => Some(CandidateRank::Bootstrap),
        ElectionEligibility::Ineligible(_) => None,
    }
}

fn compare_candidate_rank(
    left: Option<CandidateRank>,
    left_id: &MemberId,
    right: Option<CandidateRank>,
    right_id: &MemberId,
) -> Ordering {
    match (left, right) {
        (Some(CandidateRank::Promote(left_pos)), Some(CandidateRank::Promote(right_pos))) => {
            left_pos.cmp(&right_pos).then_with(|| right_id.cmp(left_id))
        }
        (Some(CandidateRank::Promote(_)), Some(CandidateRank::Bootstrap)) => Ordering::Greater,
        (Some(CandidateRank::Bootstrap), Some(CandidateRank::Promote(_))) => Ordering::Less,
        (Some(CandidateRank::Bootstrap), Some(CandidateRank::Bootstrap)) => right_id.cmp(left_id),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}

fn classify_candidate(peer: &PeerKnowledge) -> Option<()> {
    match &peer.election {
        ElectionEligibility::BootstrapEligible | ElectionEligibility::PromoteEligible(_) => {
            Some(())
        }
        ElectionEligibility::Ineligible(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{best_failover_candidate, decide};
    use crate::{
        dcs::state::DcsTrust,
        state::{MemberId, UnixMillis},
    };

    use super::super::types::{
        ApiVisibility, AuthorityView, Candidacy, DataDirState, DesiredState, ElectionEligibility,
        GlobalKnowledge, LocalDataState, LocalKnowledge, NoPrimaryReason, ObservationState,
        PeerKnowledge, PostgresState, ProcessState, PublicationState, ReplicationState,
        StorageState, SwitchoverState, TargetRole, WalPosition, WorldView,
    };

    fn promote_peer(lsn: u64) -> PeerKnowledge {
        PeerKnowledge {
            election: ElectionEligibility::PromoteEligible(WalPosition { timeline: 1, lsn }),
            api: ApiVisibility::Reachable,
        }
    }

    #[test]
    fn best_failover_candidate_includes_self_in_ranking() {
        let self_id = MemberId("node-a".to_string());
        let mut peers = BTreeMap::new();
        peers.insert(MemberId("node-b".to_string()), promote_peer(10));

        assert_eq!(
            best_failover_candidate(&peers, &promote_peer(20), &self_id),
            Some(self_id)
        );
    }

    #[test]
    fn best_failover_candidate_prefers_higher_ranked_peer() {
        let self_id = MemberId("node-a".to_string());
        let peer_id = MemberId("node-b".to_string());
        let mut peers = BTreeMap::new();
        peers.insert(peer_id.clone(), promote_peer(20));

        assert_eq!(
            best_failover_candidate(&peers, &promote_peer(10), &self_id),
            Some(peer_id)
        );
    }

    #[test]
    fn stale_observed_lease_does_not_block_failover_candidacy() {
        let self_id = MemberId("node-a".to_string());
        let stale_epoch = super::super::types::LeaseEpoch {
            holder: MemberId("node-b".to_string()),
            generation: 7,
        };
        let world = WorldView {
            local: LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Replica {
                    upstream: Some(MemberId("node-b".to_string())),
                    replication: ReplicationState::Streaming(WalPosition {
                        timeline: 1,
                        lsn: 42,
                    }),
                },
                process: ProcessState::Idle,
                storage: StorageState::Healthy,
                required_roles_ready: false,
                publication: PublicationState {
                    authority: AuthorityView::NoPrimary(NoPrimaryReason::LeaseOpen),
                    fence_cutoff: None,
                },
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                },
            },
            global: GlobalKnowledge {
                dcs_trust: DcsTrust::FullQuorum,
                lease: super::super::types::LeaseState::Unheld,
                observed_lease: Some(stale_epoch),
                observed_primary: None,
                switchover: SwitchoverState::None,
                peers: BTreeMap::new(),
                self_peer: promote_peer(42),
            },
        };

        assert_eq!(
            decide(&world, &self_id),
            DesiredState {
                role: TargetRole::Candidate(Candidacy::Failover),
                publication: super::super::types::PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::LeaseOpen,
                    fence_cutoff: None,
                },
                clear_switchover: false,
            }
        );
    }
}
