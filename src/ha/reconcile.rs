use crate::{
    pginfo::state::{PgInfoState, SqlStatus},
    process::jobs::{
        ActiveJobKind, PostgresStartIntent, ProcessIntent, ReplicaProvisionIntent, ShutdownMode,
    },
};

use super::types::{
    FollowRecovery, HaDecision, HaMode, HaObservation, HaPlan, HaStep, LocalDataState,
    PublicationState, SwitchoverState,
};

pub(crate) fn reconcile(observation: &HaObservation, decision: &HaDecision) -> HaPlan {
    let mut steps = Vec::new();

    if let Some(projection) = &decision.publication {
        if observation.publication != PublicationState::Projected(projection.clone()) {
            steps.push(HaStep::Publish(projection.clone()));
        }
    }

    if decision.clear_switchover
        && observation
            .dcs
            .switchover()
            .is_some_and(|switchover| !matches!(switchover, SwitchoverState::None))
    {
        steps.push(HaStep::ClearSwitchover);
    }

    if observation.process.active_job().is_some() {
        return steps;
    }

    let waiting_for_demote = observation
        .process
        .waiting_for_pg_observation(&observation.pg, ActiveJobKind::Demote);
    let waiting_for_start_primary = observation
        .process
        .waiting_for_pg_observation(&observation.pg, ActiveJobKind::StartPrimary);
    let waiting_for_promote = observation
        .process
        .waiting_for_pg_observation(&observation.pg, ActiveJobKind::Promote);
    let waiting_for_start_replica = observation
        .process
        .waiting_for_pg_observation(&observation.pg, ActiveJobKind::StartReplica);

    match &decision.mode {
        HaMode::Lead(_) => match observation.local_data {
            LocalDataState::Missing | LocalDataState::BootstrapEmpty => {
                steps.push(HaStep::RunProcess(ProcessIntent::Bootstrap));
            }
            LocalDataState::ConsistentReplica
            | LocalDataState::DivergedRewind
            | LocalDataState::DivergedBasebackup
                if waiting_for_start_primary || waiting_for_promote => {}
            LocalDataState::ConsistentReplica
            | LocalDataState::DivergedRewind
            | LocalDataState::DivergedBasebackup => match pg_role(&observation.pg) {
                PgRole::Offline => steps.push(HaStep::RunProcess(ProcessIntent::Start(
                    PostgresStartIntent::Primary,
                ))),
                PgRole::Replica => steps.push(HaStep::RunProcess(ProcessIntent::Promote)),
                PgRole::Primary => {
                    if !observation.managed_roles_reconciled {
                        steps.push(HaStep::ReconcileManagedRoles);
                    }
                }
            },
        },
        HaMode::AcquireLease(kind) => steps.push(HaStep::AcquireLease(kind.clone())),
        HaMode::Follow { leader, recovery } => match recovery {
            FollowRecovery::None => {}
            FollowRecovery::Basebackup | FollowRecovery::Rewind => match pg_role(&observation.pg) {
                PgRole::Primary | PgRole::Replica if waiting_for_demote => {}
                PgRole::Primary | PgRole::Replica => {
                    steps.push(HaStep::RunProcess(ProcessIntent::Demote(
                        ShutdownMode::Fast,
                    )));
                }
                PgRole::Offline => {
                    steps.push(HaStep::RunProcess(ProcessIntent::ProvisionReplica(
                        match recovery {
                            FollowRecovery::Basebackup => ReplicaProvisionIntent::BaseBackup {
                                leader: leader.clone(),
                            },
                            FollowRecovery::Rewind => ReplicaProvisionIntent::PgRewind {
                                leader: leader.clone(),
                            },
                            FollowRecovery::None | FollowRecovery::StartStreaming => unreachable!(),
                        },
                    )));
                }
            },
            FollowRecovery::StartStreaming => {
                if waiting_for_start_replica || waiting_for_demote {
                    return steps;
                }

                match pg_role(&observation.pg) {
                    PgRole::Offline => steps.push(HaStep::RunProcess(ProcessIntent::Start(
                        PostgresStartIntent::Replica {
                            leader: leader.clone(),
                        },
                    ))),
                    PgRole::Primary => {
                        steps.push(HaStep::RunProcess(ProcessIntent::Demote(
                            ShutdownMode::Fast,
                        )));
                    }
                    PgRole::Replica if observation.resolved_upstream.as_ref() == Some(leader) => {}
                    PgRole::Replica => {
                        steps.push(HaStep::RunProcess(ProcessIntent::Demote(
                            ShutdownMode::Fast,
                        )));
                    }
                }
            }
        },
        HaMode::FailsafeStop { shutdown, .. } => match pg_role(&observation.pg) {
            PgRole::Primary | PgRole::Replica if waiting_for_demote => {}
            PgRole::Primary | PgRole::Replica => {
                steps.push(HaStep::RunProcess(ProcessIntent::Demote(shutdown.clone())));
            }
            PgRole::Offline => {}
        },
        HaMode::FailsafeKeepFollowing { .. } => {}
        HaMode::WaitForQuorum => match pg_role(&observation.pg) {
            PgRole::Primary if waiting_for_demote => {}
            PgRole::Primary => {
                steps.push(HaStep::RunProcess(ProcessIntent::Demote(
                    ShutdownMode::Immediate,
                )));
            }
            PgRole::Offline | PgRole::Replica => {}
        },
        HaMode::WaitForLeader | HaMode::WaitForTarget(_) => match pg_role(&observation.pg) {
            PgRole::Primary if waiting_for_demote => {}
            PgRole::Primary => {
                steps.push(HaStep::RunProcess(ProcessIntent::Demote(
                    ShutdownMode::Fast,
                )));
            }
            PgRole::Offline if !matches!(observation.local_data, LocalDataState::Missing) => {
                steps.push(HaStep::RunProcess(ProcessIntent::Start(
                    PostgresStartIntent::DetachedStandby,
                )));
            }
            PgRole::Offline | PgRole::Replica => {}
        },
        HaMode::DemoteForSwitchover(_) => match pg_role(&observation.pg) {
            PgRole::Primary | PgRole::Replica if waiting_for_demote => {}
            PgRole::Primary | PgRole::Replica => {
                steps.push(HaStep::RunProcess(ProcessIntent::Demote(
                    ShutdownMode::Fast,
                )));
            }
            PgRole::Offline => steps.push(HaStep::ReleaseLease),
        },
        HaMode::Fence {
            release_lease,
            shutdown,
        } => {
            if *release_lease {
                steps.push(HaStep::ReleaseLease);
            } else if let Some(mode) = shutdown {
                match pg_role(&observation.pg) {
                    PgRole::Primary | PgRole::Replica if waiting_for_demote => {}
                    PgRole::Primary | PgRole::Replica => {
                        steps.push(HaStep::RunProcess(ProcessIntent::Demote(mode.clone())));
                    }
                    PgRole::Offline => {}
                }
            }
        }
    }

    steps
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PgRole {
    Offline,
    Primary,
    Replica,
}

fn pg_role(pg: &PgInfoState) -> PgRole {
    match pg {
        PgInfoState::Primary { common, .. } if common.sql == SqlStatus::Healthy => PgRole::Primary,
        PgInfoState::Replica { common, .. } if common.sql == SqlStatus::Healthy => PgRole::Replica,
        PgInfoState::Unknown { .. } | PgInfoState::Primary { .. } | PgInfoState::Replica { .. } => {
            PgRole::Offline
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        dcs::DcsSnapshot,
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{LeaseEpoch, MemberId, SwitchoverState, TimelineId, WalLsn, WorkerStatus},
    };

    use super::*;
    use crate::ha::types::{AuthorityProjection, NoPrimaryProjection};

    fn common() -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
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

    fn primary() -> PgInfoState {
        PgInfoState::Primary {
            common: common(),
            wal_lsn: WalLsn(42),
            slots: Vec::new(),
        }
    }

    fn replica() -> PgInfoState {
        PgInfoState::Replica {
            common: common(),
            replay_lsn: WalLsn(42),
            follow_lsn: Some(WalLsn(42)),
            upstream: None,
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
            self_candidate: crate::ha::types::CandidateState::Ineligible,
            storage_stalled: false,
            ready_primary: None,
        }
    }

    #[test]
    fn publication_change_is_emitted_as_first_step() {
        let steps = reconcile(
            &observation(PgInfoState::unknown(
                WorkerStatus::Running,
                SqlStatus::Unknown,
                None,
            )),
            &HaDecision {
                mode: HaMode::WaitForLeader,
                publication: Some(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::LeaseOpen,
                )),
                clear_switchover: false,
            },
        );

        assert_eq!(
            steps,
            vec![
                HaStep::Publish(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::LeaseOpen
                )),
                HaStep::RunProcess(ProcessIntent::Start(PostgresStartIntent::DetachedStandby,)),
            ]
        );
    }

    #[test]
    fn demoting_for_switchover_releases_lease_once_offline() {
        let steps = reconcile(
            &observation(PgInfoState::unknown(
                WorkerStatus::Running,
                SqlStatus::Unknown,
                None,
            )),
            &HaDecision {
                mode: HaMode::DemoteForSwitchover(MemberId("node-b".to_string())),
                publication: None,
                clear_switchover: false,
            },
        );

        assert_eq!(steps, vec![HaStep::ReleaseLease]);
    }

    #[test]
    fn matching_projection_does_not_republish() {
        let mut observation = observation(PgInfoState::unknown(
            WorkerStatus::Running,
            SqlStatus::Unknown,
            None,
        ));
        observation.publication = PublicationState::Projected(AuthorityProjection::NoPrimary(
            NoPrimaryProjection::LeaseOpen,
        ));

        assert_eq!(
            reconcile(
                &observation,
                &HaDecision {
                    mode: HaMode::WaitForLeader,
                    publication: Some(AuthorityProjection::NoPrimary(
                        NoPrimaryProjection::LeaseOpen,
                    )),
                    clear_switchover: false,
                },
            ),
            vec![HaStep::RunProcess(ProcessIntent::Start(
                PostgresStartIntent::DetachedStandby,
            ))]
        );
    }

    #[test]
    fn follower_replica_without_matching_upstream_is_restarted() {
        let observation = observation(replica());

        assert_eq!(
            reconcile(
                &observation,
                &HaDecision {
                    mode: HaMode::Follow {
                        leader: MemberId("node-b".to_string()),
                        recovery: FollowRecovery::StartStreaming,
                    },
                    publication: None,
                    clear_switchover: false,
                },
            ),
            vec![HaStep::RunProcess(ProcessIntent::Demote(
                ShutdownMode::Fast
            ))]
        );
    }

    #[test]
    fn active_process_blocks_new_process_steps() {
        let mut observation = observation(primary());
        observation.process = ProcessState::Running {
            worker: WorkerStatus::Running,
            active: crate::process::jobs::ActiveJob {
                id: crate::state::JobId("job-1".to_string()),
                kind: ActiveJobKind::Promote,
                started_at: crate::state::UnixMillis(10),
                deadline_at: crate::state::UnixMillis(20),
            },
        };

        assert_eq!(
            reconcile(
                &observation,
                &HaDecision {
                    mode: HaMode::Lead(LeaseEpoch {
                        holder: MemberId("node-a".to_string()),
                        generation: 1,
                    }),
                    publication: None,
                    clear_switchover: false,
                },
            ),
            Vec::<HaStep>::new()
        );
    }
}
