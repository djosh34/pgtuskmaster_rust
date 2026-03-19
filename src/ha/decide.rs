use std::cmp::Ordering;

use crate::{
    dcs::{DcsMemberState, DcsQuorumState},
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    process::jobs::ShutdownMode,
    state::{LeaseEpoch, MemberId, ObservedWalPosition, SwitchoverState},
};

use super::types::{
    AuthorityProjection, CandidateState, FenceCutoff, FollowRecovery, HaDecision, HaMode,
    HaObservation, LeaseClaim, LocalDataState, NoPrimaryFence, NoPrimaryProjection,
    PublicationState,
};

pub(crate) fn decide(observation: &HaObservation, self_id: &MemberId) -> HaDecision {
    let healthy_primary = matches!(
        observation.pg,
        PgInfoState::Primary {
            common: crate::pginfo::state::PgInfoCommon {
                sql: SqlStatus::Healthy,
                ..
            },
            ..
        }
    );
    let healthy_replica = matches!(
        observation.pg,
        PgInfoState::Replica {
            common: crate::pginfo::state::PgInfoCommon {
                sql: SqlStatus::Healthy,
                ..
            },
            ..
        }
    );

    if !observation.dcs.is_quorum() {
        let publication = Some(AuthorityProjection::NoPrimary(
            NoPrimaryProjection::NoQuorum {
                fence: match (
                    publication_epoch(&observation.publication),
                    committed_lsn(&observation.pg),
                ) {
                    (Some(epoch), Some(committed_lsn)) => NoPrimaryFence::Cutoff(FenceCutoff {
                        epoch,
                        committed_lsn,
                    }),
                    (None, _) | (_, None) => NoPrimaryFence::None,
                },
            },
        ));

        return match &observation.pg {
            PgInfoState::Primary { .. }
                if publication_epoch(&observation.publication).is_some() && healthy_primary =>
            {
                HaDecision {
                    mode: HaMode::FailsafeStop {
                        shutdown: ShutdownMode::Immediate,
                        cutoff: match (
                            publication_epoch(&observation.publication),
                            committed_lsn(&observation.pg),
                        ) {
                            (Some(epoch), Some(committed_lsn)) => Some(FenceCutoff {
                                epoch,
                                committed_lsn,
                            }),
                            (None, _) | (_, None) => None,
                        },
                    },
                    publication,
                    clear_switchover: false,
                }
            }
            PgInfoState::Replica { .. } if healthy_replica => HaDecision {
                mode: HaMode::FailsafeKeepFollowing {
                    leader: observation.resolved_upstream.clone(),
                },
                publication,
                clear_switchover: false,
            },
            PgInfoState::Unknown { .. }
            | PgInfoState::Primary { .. }
            | PgInfoState::Replica { .. } => HaDecision {
                mode: HaMode::WaitForQuorum,
                publication,
                clear_switchover: false,
            },
        };
    }

    let Some(quorum) = observation.dcs.quorum_state() else {
        unreachable!("quorum checked above");
    };

    if observation.storage_stalled && healthy_primary {
        let active_epoch = quorum.leadership.clone();
        return HaDecision {
            mode: HaMode::Fence {
                release_lease: quorum
                    .leadership
                    .as_ref()
                    .is_some_and(|epoch| epoch.holder == *self_id),
                shutdown: Some(ShutdownMode::Immediate),
            },
            publication: Some(AuthorityProjection::NoPrimary(
                NoPrimaryProjection::Recovering {
                    epoch: active_epoch.clone(),
                    fence: match (active_epoch, committed_lsn(&observation.pg)) {
                        (Some(epoch), Some(committed_lsn)) => NoPrimaryFence::Cutoff(FenceCutoff {
                            epoch,
                            committed_lsn,
                        }),
                        (None, _) | (_, None) => NoPrimaryFence::None,
                    },
                },
            )),
            clear_switchover: false,
        };
    }

    match quorum.leadership.as_ref() {
        Some(epoch) if epoch.holder == *self_id => {
            let publication = match &observation.pg {
                PgInfoState::Primary { common, .. } if common.sql == SqlStatus::Healthy => {
                    Some(AuthorityProjection::Primary(epoch.clone()))
                }
                PgInfoState::Unknown { .. }
                | PgInfoState::Primary { .. }
                | PgInfoState::Replica { .. } => Some(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::Recovering {
                        epoch: Some(epoch.clone()),
                        fence: NoPrimaryFence::None,
                    },
                )),
            };

            match &quorum.switchover {
                SwitchoverState::None => HaDecision {
                    mode: HaMode::Lead(epoch.clone()),
                    publication,
                    clear_switchover: false,
                },
                SwitchoverState::Specific(target) if target == self_id => HaDecision {
                    mode: HaMode::Lead(epoch.clone()),
                    publication,
                    clear_switchover: false,
                },
                SwitchoverState::Specific(target)
                    if quorum
                        .member(target)
                        .is_some_and(|member| candidate_for_member(member).is_eligible()) =>
                {
                    HaDecision {
                        mode: HaMode::DemoteForSwitchover(target.clone()),
                        publication: None,
                        clear_switchover: false,
                    }
                }
                SwitchoverState::Specific(_) => HaDecision {
                    mode: HaMode::Lead(epoch.clone()),
                    publication,
                    clear_switchover: true,
                },
                SwitchoverState::AnyHealthyReplica => {
                    let best_target = quorum
                        .members()
                        .filter(|(member_id, _)| *member_id != self_id)
                        .filter_map(|(member_id, member)| {
                            let candidate = candidate_for_member(member);
                            candidate
                                .is_eligible()
                                .then_some((member_id.clone(), candidate))
                        })
                        .max_by(|(left_id, left), (right_id, right)| {
                            compare_candidates(left_id, left, right_id, right)
                        })
                        .map(|(member_id, _)| member_id);

                    match best_target {
                        Some(target) => HaDecision {
                            mode: HaMode::DemoteForSwitchover(target),
                            publication: None,
                            clear_switchover: false,
                        },
                        None => HaDecision {
                            mode: HaMode::Lead(epoch.clone()),
                            publication,
                            clear_switchover: false,
                        },
                    }
                }
            }
        }
        Some(epoch) => {
            let leader_ready = quorum.member(&epoch.holder).is_some_and(|member| {
                matches!(member.postgres(), PgInfoState::Primary { .. })
                    && member.postgres().readiness() == Readiness::Ready
            });
            let publication = Some(if leader_ready {
                AuthorityProjection::Primary(epoch.clone())
            } else {
                AuthorityProjection::NoPrimary(NoPrimaryProjection::Recovering {
                    epoch: Some(epoch.clone()),
                    fence: NoPrimaryFence::None,
                })
            });

            match &observation.pg {
                PgInfoState::Primary { common, .. } if common.sql == SqlStatus::Healthy => {
                    HaDecision {
                        mode: HaMode::Fence {
                            release_lease: false,
                            shutdown: Some(ShutdownMode::Immediate),
                        },
                        publication,
                        clear_switchover: false,
                    }
                }
                PgInfoState::Unknown { .. }
                | PgInfoState::Primary { .. }
                | PgInfoState::Replica { .. }
                    if leader_ready =>
                {
                    HaDecision {
                        mode: HaMode::Follow {
                            leader: epoch.holder.clone(),
                            recovery: follow_recovery(observation, &epoch.holder),
                        },
                        publication,
                        clear_switchover: false,
                    }
                }
                PgInfoState::Unknown { .. }
                | PgInfoState::Primary { .. }
                | PgInfoState::Replica { .. } => HaDecision {
                    mode: HaMode::WaitForLeader,
                    publication,
                    clear_switchover: false,
                },
            }
        }
        None => {
            let publication = Some(AuthorityProjection::NoPrimary(
                NoPrimaryProjection::LeaseOpen,
            ));
            match &quorum.switchover {
                SwitchoverState::Specific(target)
                    if target == self_id && observation.self_candidate.is_eligible() =>
                {
                    HaDecision {
                        mode: HaMode::AcquireLease(LeaseClaim::TargetedSwitchover(target.clone())),
                        publication,
                        clear_switchover: false,
                    }
                }
                SwitchoverState::Specific(target)
                    if quorum
                        .member(target)
                        .is_some_and(|member| candidate_for_member(member).is_eligible()) =>
                {
                    HaDecision {
                        mode: HaMode::WaitForTarget(target.clone()),
                        publication,
                        clear_switchover: false,
                    }
                }
                SwitchoverState::Specific(_) => HaDecision {
                    mode: HaMode::WaitForLeader,
                    publication,
                    clear_switchover: true,
                },
                SwitchoverState::AnyHealthyReplica => {
                    let best =
                        best_failover_candidate(quorum, &observation.self_candidate, self_id);
                    match best {
                        Some(candidate) if candidate == *self_id => HaDecision {
                            mode: HaMode::AcquireLease(lease_claim(observation, self_id)),
                            publication,
                            clear_switchover: false,
                        },
                        Some(candidate) => HaDecision {
                            mode: HaMode::WaitForTarget(candidate),
                            publication,
                            clear_switchover: false,
                        },
                        None => HaDecision {
                            mode: HaMode::WaitForLeader,
                            publication,
                            clear_switchover: false,
                        },
                    }
                }
                SwitchoverState::None => {
                    match best_failover_candidate(quorum, &observation.self_candidate, self_id) {
                        Some(candidate) if candidate == *self_id => HaDecision {
                            mode: HaMode::AcquireLease(lease_claim(observation, self_id)),
                            publication,
                            clear_switchover: false,
                        },
                        Some(_) | None => HaDecision {
                            mode: HaMode::WaitForLeader,
                            publication,
                            clear_switchover: false,
                        },
                    }
                }
            }
        }
    }
}

fn committed_lsn(pg: &PgInfoState) -> Option<u64> {
    match pg {
        PgInfoState::Primary {
            common, wal_lsn, ..
        } if common.sql == SqlStatus::Healthy => Some(wal_lsn.0),
        PgInfoState::Unknown { .. } | PgInfoState::Primary { .. } | PgInfoState::Replica { .. } => {
            None
        }
    }
}

fn follow_recovery(observation: &HaObservation, leader: &MemberId) -> FollowRecovery {
    match observation.local_data {
        LocalDataState::Missing | LocalDataState::BootstrapEmpty => FollowRecovery::Basebackup,
        LocalDataState::ConsistentReplica => match &observation.pg {
            PgInfoState::Replica { common, .. }
                if common.sql == SqlStatus::Healthy
                    && observation.resolved_upstream.as_ref() == Some(leader) =>
            {
                FollowRecovery::None
            }
            PgInfoState::Unknown { .. }
            | PgInfoState::Primary { .. }
            | PgInfoState::Replica { .. } => {
                if observation.process.rewind_failed_requires_basebackup() {
                    FollowRecovery::Basebackup
                } else {
                    FollowRecovery::StartStreaming
                }
            }
        },
        LocalDataState::DivergedRewind => {
            if observation.process.rewind_failed_requires_basebackup() {
                FollowRecovery::Basebackup
            } else if observation
                .process
                .basebackup_completed_awaiting_pg_start(&observation.pg)
            {
                FollowRecovery::StartStreaming
            } else {
                FollowRecovery::Rewind
            }
        }
        LocalDataState::DivergedBasebackup => FollowRecovery::Basebackup,
    }
}

fn lease_claim(observation: &HaObservation, self_id: &MemberId) -> LeaseClaim {
    if matches!(
        observation.local_data,
        LocalDataState::Missing | LocalDataState::BootstrapEmpty
    ) {
        return LeaseClaim::Bootstrap;
    }

    if matches!(
        observation.publication,
        PublicationState::Projected(AuthorityProjection::NoPrimary(
            NoPrimaryProjection::NoQuorum { .. }
        ))
    ) {
        return LeaseClaim::ResumeAfterOutage;
    }

    if observation.dcs.switchover().is_some_and(
        |switchover| matches!(switchover, SwitchoverState::Specific(target) if target == self_id),
    ) {
        return LeaseClaim::TargetedSwitchover(self_id.clone());
    }

    LeaseClaim::Failover
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
            | NoPrimaryProjection::Recovering { epoch: None, .. },
        )) => None,
    }
}

fn candidate_for_member(member: &DcsMemberState) -> CandidateState {
    if member.postgres().readiness() != Readiness::Ready {
        return CandidateState::Ineligible;
    }

    match member.postgres() {
        PgInfoState::Unknown { .. } => CandidateState::Bootstrap,
        PgInfoState::Primary { .. } => member
            .postgres()
            .committed_wal()
            .map(CandidateState::Promote)
            .unwrap_or(CandidateState::Ineligible),
        PgInfoState::Replica { .. } => member
            .postgres()
            .replay_wal()
            .or_else(|| member.postgres().follow_wal())
            .map(CandidateState::Promote)
            .unwrap_or(CandidateState::Ineligible),
    }
}

fn compare_candidates(
    left_id: &MemberId,
    left: &CandidateState,
    right_id: &MemberId,
    right: &CandidateState,
) -> Ordering {
    match (left, right) {
        (CandidateState::Promote(left_pos), CandidateState::Promote(right_pos)) => {
            compare_positions(left_pos, right_pos).then_with(|| right_id.cmp(left_id))
        }
        (CandidateState::Promote(_), CandidateState::Bootstrap) => Ordering::Greater,
        (CandidateState::Bootstrap, CandidateState::Promote(_)) => Ordering::Less,
        (CandidateState::Bootstrap, CandidateState::Bootstrap) => right_id.cmp(left_id),
        (CandidateState::Ineligible, CandidateState::Ineligible) => Ordering::Equal,
        (CandidateState::Ineligible, CandidateState::Bootstrap | CandidateState::Promote(_)) => {
            Ordering::Less
        }
        (CandidateState::Bootstrap | CandidateState::Promote(_), CandidateState::Ineligible) => {
            Ordering::Greater
        }
    }
}

fn compare_positions(left: &ObservedWalPosition, right: &ObservedWalPosition) -> Ordering {
    left.timeline
        .map(|value| value.0)
        .unwrap_or_default()
        .cmp(&right.timeline.map(|value| value.0).unwrap_or_default())
        .then_with(|| left.lsn.0.cmp(&right.lsn.0))
}

fn best_failover_candidate(
    quorum: &DcsQuorumState,
    self_candidate: &CandidateState,
    self_id: &MemberId,
) -> Option<MemberId> {
    let best_peer = quorum
        .members()
        .filter(|(member_id, _)| *member_id != self_id)
        .filter_map(|(member_id, member)| {
            let candidate = candidate_for_member(member);
            candidate
                .is_eligible()
                .then_some((member_id.clone(), candidate))
        })
        .max_by(|(left_id, left), (right_id, right)| {
            compare_candidates(left_id, left, right_id, right)
        });

    if !self_candidate.is_eligible() {
        return best_peer.map(|(member_id, _)| member_id);
    }

    match best_peer {
        Some((member_id, candidate))
            if compare_candidates(self_id, self_candidate, &member_id, &candidate)
                == Ordering::Greater =>
        {
            Some(self_id.clone())
        }
        Some((member_id, _)) => Some(member_id),
        None => Some(self_id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        dcs::{DcsMemberState, DcsSnapshot},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{
            LeaseEpoch, MemberId, ObservedWalPosition, PgEndpoint, SwitchoverState, TimelineId,
            WalLsn, WorkerStatus,
        },
    };

    use super::*;

    fn common(readiness: Readiness) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql: SqlStatus::Healthy,
            readiness,
            timeline: Some(TimelineId(1)),
            system_identifier: None,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: None,
        }
    }

    fn primary(lsn: u64) -> PgInfoState {
        PgInfoState::Primary {
            common: common(Readiness::Ready),
            wal_lsn: WalLsn(lsn),
            slots: Vec::new(),
        }
    }

    fn replica(lsn: u64) -> PgInfoState {
        PgInfoState::Replica {
            common: common(Readiness::Ready),
            replay_lsn: WalLsn(lsn),
            follow_lsn: Some(WalLsn(lsn)),
            upstream: None,
        }
    }

    fn peer(member_id: &str, pg: PgInfoState) -> DcsMemberState {
        DcsMemberState {
            postgres_endpoint: PgEndpoint::Tcp {
                host: member_id.to_string(),
                port: 5432,
            },
            postgres: pg,
        }
    }

    fn observation(pg: PgInfoState) -> HaObservation {
        HaObservation {
            pg,
            process: ProcessState::Idle {
                worker: WorkerStatus::Running,
                last_outcome: None,
            },
            dcs: DcsSnapshot::quorum(None, SwitchoverState::None, BTreeMap::new()),
            publication: PublicationState::unknown(),
            managed_roles_reconciled: false,
            local_data: LocalDataState::ConsistentReplica,
            resolved_upstream: None,
            self_candidate: CandidateState::Promote(ObservedWalPosition {
                timeline: Some(TimelineId(1)),
                lsn: WalLsn(10),
            }),
            storage_stalled: false,
            ready_primary: None,
        }
    }

    #[test]
    fn best_failover_candidate_includes_self_in_ranking() {
        let self_id = MemberId("node-a".to_string());
        let quorum = crate::dcs::DcsQuorumState {
            leadership: None,
            switchover: SwitchoverState::None,
            members: BTreeMap::from([(
                MemberId("node-b".to_string()),
                peer("node-b", replica(20)),
            )]),
        };

        assert_eq!(
            best_failover_candidate(
                &quorum,
                &CandidateState::Promote(ObservedWalPosition {
                    timeline: Some(TimelineId(1)),
                    lsn: WalLsn(30),
                }),
                &self_id,
            ),
            Some(self_id)
        );
    }

    #[test]
    fn no_quorum_keeps_replica_in_failsafe() {
        let self_id = MemberId("node-a".to_string());
        let mut observation = observation(replica(42));
        observation.resolved_upstream = Some(MemberId("node-b".to_string()));
        observation.dcs = DcsSnapshot::NoQuorum;

        assert_eq!(
            decide(&observation, &self_id),
            HaDecision {
                mode: HaMode::FailsafeKeepFollowing {
                    leader: Some(MemberId("node-b".to_string())),
                },
                publication: Some(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::NoQuorum {
                        fence: NoPrimaryFence::None,
                    }
                )),
                clear_switchover: false,
            }
        );
    }

    #[test]
    fn basebackup_completion_on_diverged_data_transitions_to_start_streaming() {
        let self_id = MemberId("node-b".to_string());
        let epoch = LeaseEpoch {
            holder: MemberId("node-a".to_string()),
            generation: 7,
        };
        let mut observation = observation(PgInfoState::unknown(
            WorkerStatus::Running,
            SqlStatus::Unknown,
            None,
        ));
        observation.local_data = LocalDataState::DivergedRewind;
        observation.process = ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: Some(crate::process::state::JobOutcome::Success {
                id: crate::state::JobId("job-1".to_string()),
                job_kind: crate::process::jobs::ActiveJobKind::BaseBackup,
                finished_at: crate::state::UnixMillis(20),
            }),
        };
        observation.dcs = DcsSnapshot::quorum(
            Some(epoch.clone()),
            SwitchoverState::None,
            BTreeMap::from([(epoch.holder.clone(), peer("node-a", primary(50)))]),
        );

        assert_eq!(
            decide(&observation, &self_id).mode,
            HaMode::Follow {
                leader: MemberId("node-a".to_string()),
                recovery: FollowRecovery::StartStreaming,
            }
        );
    }

    #[test]
    fn generic_switchover_waits_for_future_target() {
        let self_id = MemberId("node-a".to_string());
        let mut observation = observation(primary(42));
        observation.dcs = DcsSnapshot::quorum(
            Some(LeaseEpoch {
                holder: self_id.clone(),
                generation: 7,
            }),
            SwitchoverState::AnyHealthyReplica,
            BTreeMap::from([(
                MemberId("node-b".to_string()),
                peer(
                    "node-b",
                    PgInfoState::Unknown {
                        common: common(Readiness::NotReady),
                    },
                ),
            )]),
        );

        assert_eq!(
            decide(&observation, &self_id).mode,
            HaMode::Lead(LeaseEpoch {
                holder: self_id,
                generation: 7,
            })
        );
    }

    #[test]
    fn lease_holder_demotes_for_best_switchover_target() {
        let self_id = MemberId("node-c".to_string());
        let mut observation = observation(replica(50));
        observation.dcs = DcsSnapshot::quorum(
            Some(LeaseEpoch {
                holder: self_id.clone(),
                generation: 7,
            }),
            SwitchoverState::AnyHealthyReplica,
            BTreeMap::from([(MemberId("node-a".to_string()), peer("node-a", replica(40)))]),
        );

        assert_eq!(
            decide(&observation, &self_id).mode,
            HaMode::DemoteForSwitchover(MemberId("node-a".to_string()))
        );
    }
}
