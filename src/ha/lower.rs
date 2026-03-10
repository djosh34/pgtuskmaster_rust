use serde::{Deserialize, Serialize};

use crate::{dcs::state::MemberStateClass, process::jobs::ActiveJobKind, state::MemberId};

use super::{
    decision::{process_activity, ProcessActivity, ReconcileFacts},
    state::{DesiredNodeState, FencePlan, PrimaryPlan, ReplicaPlan},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HaEffectPlan {
    pub(crate) lease: LeaseEffect,
    pub(crate) switchover: SwitchoverEffect,
    pub(crate) recovery: RecoveryEffect,
    pub(crate) postgres: PostgresEffect,
    pub(crate) safety: SafetyEffect,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LeaseEffect {
    #[default]
    None,
    AcquireLeader,
    ReleaseLeader,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SwitchoverEffect {
    #[default]
    None,
    ClearRequest,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum RecoveryEffect {
    #[default]
    None,
    Rewind {
        leader_member_id: MemberId,
    },
    Basebackup {
        leader_member_id: MemberId,
    },
    Bootstrap,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PostgresEffect {
    #[default]
    None,
    StartPrimary,
    StartReplica {
        leader_member_id: MemberId,
    },
    Promote,
    Demote,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SafetyEffect {
    #[default]
    None,
    FenceNode,
}

pub(crate) fn lower_desired_state(
    desired_state: &DesiredNodeState,
    facts: &ReconcileFacts,
) -> HaEffectPlan {
    match desired_state {
        DesiredNodeState::Bootstrap { .. } => lower_bootstrap(facts),
        DesiredNodeState::Primary { plan } => lower_primary(plan, facts),
        DesiredNodeState::Replica { plan } => lower_replica(plan, facts),
        DesiredNodeState::Quiescent { .. } => lower_quiescent(facts),
        DesiredNodeState::Fence { plan } => lower_fence(plan, facts),
    }
}

fn lower_bootstrap(facts: &ReconcileFacts) -> HaEffectPlan {
    match process_activity(&facts.current_process, &[ActiveJobKind::Bootstrap]) {
        ProcessActivity::Running => HaEffectPlan::default(),
        _ => HaEffectPlan {
            recovery: RecoveryEffect::Bootstrap,
            ..HaEffectPlan::default()
        },
    }
}

fn lower_primary(plan: &PrimaryPlan, facts: &ReconcileFacts) -> HaEffectPlan {
    match plan {
        PrimaryPlan::KeepLeader => maybe_clear_switchover(facts, HaEffectPlan::default()),
        PrimaryPlan::AcquireLeaderThenResumePrimary => maybe_clear_switchover(
            facts,
            HaEffectPlan {
                lease: if facts.i_am_authoritative_leader {
                    LeaseEffect::None
                } else {
                    LeaseEffect::AcquireLeader
                },
                ..HaEffectPlan::default()
            },
        ),
        PrimaryPlan::AcquireLeaderThenPromote => HaEffectPlan {
            lease: if facts.i_am_authoritative_leader {
                LeaseEffect::None
            } else {
                LeaseEffect::AcquireLeader
            },
            postgres: match process_activity(&facts.current_process, &[ActiveJobKind::Promote]) {
                ProcessActivity::Running => PostgresEffect::None,
                _ => PostgresEffect::Promote,
            },
            ..HaEffectPlan::default()
        },
        PrimaryPlan::AcquireLeaderThenStartPrimary => HaEffectPlan {
            lease: if facts.i_am_authoritative_leader {
                LeaseEffect::None
            } else {
                LeaseEffect::AcquireLeader
            },
            postgres: match process_activity(
                &facts.current_process,
                &[ActiveJobKind::StartPostgres],
            ) {
                ProcessActivity::Running => PostgresEffect::None,
                _ => PostgresEffect::StartPrimary,
            },
            ..HaEffectPlan::default()
        },
    }
}

fn lower_replica(plan: &ReplicaPlan, facts: &ReconcileFacts) -> HaEffectPlan {
    match plan {
        ReplicaPlan::Direct { leader_member_id } => lower_direct_follow(leader_member_id, facts),
        ReplicaPlan::Rewind { leader_member_id } => {
            lower_rewind_then_follow(leader_member_id, facts)
        }
        ReplicaPlan::Basebackup { leader_member_id } => {
            lower_basebackup_then_follow(leader_member_id, facts)
        }
    }
}

fn lower_direct_follow(leader_member_id: &MemberId, facts: &ReconcileFacts) -> HaEffectPlan {
    if facts.postgres_primary {
        if facts.replica_targets_authoritative_leader == Some(true) {
            return HaEffectPlan::default();
        }
        return HaEffectPlan {
            postgres: PostgresEffect::Demote,
            ..HaEffectPlan::default()
        };
    }

    if facts.postgres_reachable {
        if facts.postgres_replica {
            if facts.replica_targets_authoritative_leader == Some(false) {
                return HaEffectPlan {
                    postgres: PostgresEffect::Demote,
                    ..HaEffectPlan::default()
                };
            }
            if facts.replica_targets_authoritative_leader.is_none() {
                return match process_activity(
                    &facts.current_process,
                    &[ActiveJobKind::StartPostgres],
                ) {
                    ProcessActivity::Running => HaEffectPlan::default(),
                    _ => HaEffectPlan {
                        postgres: PostgresEffect::StartReplica {
                            leader_member_id: leader_member_id.clone(),
                        },
                        ..HaEffectPlan::default()
                    },
                };
            }
            return maybe_clear_switchover(facts, HaEffectPlan::default());
        }
        return HaEffectPlan::default();
    }

    match process_activity(&facts.current_process, &[ActiveJobKind::StartPostgres]) {
        ProcessActivity::Running => HaEffectPlan::default(),
        _ => HaEffectPlan {
            postgres: PostgresEffect::StartReplica {
                leader_member_id: leader_member_id.clone(),
            },
            ..HaEffectPlan::default()
        },
    }
}

fn lower_rewind_then_follow(leader_member_id: &MemberId, facts: &ReconcileFacts) -> HaEffectPlan {
    if facts.postgres_primary {
        return HaEffectPlan {
            postgres: PostgresEffect::Demote,
            ..HaEffectPlan::default()
        };
    }

    match process_activity(&facts.current_process, &[ActiveJobKind::PgRewind]) {
        ProcessActivity::Running => HaEffectPlan::default(),
        ProcessActivity::IdleSuccess => lower_direct_follow(leader_member_id, facts),
        _ => HaEffectPlan {
            recovery: RecoveryEffect::Rewind {
                leader_member_id: leader_member_id.clone(),
            },
            ..HaEffectPlan::default()
        },
    }
}

fn lower_basebackup_then_follow(
    leader_member_id: &MemberId,
    facts: &ReconcileFacts,
) -> HaEffectPlan {
    if facts.postgres_primary || facts.postgres_reachable {
        return HaEffectPlan {
            postgres: PostgresEffect::Demote,
            ..HaEffectPlan::default()
        };
    }

    if !local_data_dir_is_missing_or_empty(facts) {
        match process_activity(&facts.current_process, &[ActiveJobKind::Fencing]) {
            ProcessActivity::Running => return HaEffectPlan::default(),
            ProcessActivity::IdleSuccess => {}
            ProcessActivity::IdleFailure | ProcessActivity::IdleNoOutcome => {
                return HaEffectPlan {
                    safety: SafetyEffect::FenceNode,
                    ..HaEffectPlan::default()
                };
            }
        }
    }

    match process_activity(&facts.current_process, &[ActiveJobKind::BaseBackup]) {
        ProcessActivity::Running => HaEffectPlan::default(),
        ProcessActivity::IdleSuccess => lower_direct_follow(leader_member_id, facts),
        _ => HaEffectPlan {
            recovery: RecoveryEffect::Basebackup {
                leader_member_id: leader_member_id.clone(),
            },
            ..HaEffectPlan::default()
        },
    }
}

fn local_data_dir_is_missing_or_empty(facts: &ReconcileFacts) -> bool {
    facts
        .local_member
        .as_ref()
        .and_then(|member| member.state_class.clone())
        == Some(MemberStateClass::EmptyOrMissingDataDir)
}

fn lower_quiescent(facts: &ReconcileFacts) -> HaEffectPlan {
    if facts.i_am_authoritative_leader || facts.postgres_primary {
        return HaEffectPlan {
            postgres: if facts.postgres_primary {
                PostgresEffect::Demote
            } else {
                PostgresEffect::None
            },
            lease: if facts.i_am_authoritative_leader {
                LeaseEffect::ReleaseLeader
            } else {
                LeaseEffect::None
            },
            ..HaEffectPlan::default()
        };
    }

    HaEffectPlan::default()
}

fn lower_fence(_plan: &FencePlan, facts: &ReconcileFacts) -> HaEffectPlan {
    let fencing_running = matches!(
        process_activity(&facts.current_process, &[ActiveJobKind::Fencing]),
        ProcessActivity::Running
    );
    HaEffectPlan {
        lease: if facts.i_am_authoritative_leader {
            LeaseEffect::ReleaseLeader
        } else {
            LeaseEffect::None
        },
        safety: if fencing_running {
            SafetyEffect::None
        } else {
            SafetyEffect::FenceNode
        },
        ..HaEffectPlan::default()
    }
}

fn maybe_clear_switchover(facts: &ReconcileFacts, plan: HaEffectPlan) -> HaEffectPlan {
    if facts.switchover_pending && facts.i_am_authoritative_leader && facts.postgres_primary {
        return HaEffectPlan {
            switchover: SwitchoverEffect::ClearRequest,
            ..plan
        };
    }
    plan
}

impl HaEffectPlan {
    pub(crate) fn len(&self) -> usize {
        self.dispatch_step_count()
    }

    pub(crate) fn dispatch_step_count(&self) -> usize {
        lease_effect_step_count(&self.lease)
            + switchover_effect_step_count(&self.switchover)
            + recovery_effect_step_count(&self.recovery)
            + postgres_effect_step_count(&self.postgres)
            + safety_effect_step_count(&self.safety)
    }
}

pub(crate) fn lease_effect_step_count(effect: &LeaseEffect) -> usize {
    match effect {
        LeaseEffect::None => 0,
        LeaseEffect::AcquireLeader | LeaseEffect::ReleaseLeader => 1,
    }
}

pub(crate) fn switchover_effect_step_count(effect: &SwitchoverEffect) -> usize {
    match effect {
        SwitchoverEffect::None => 0,
        SwitchoverEffect::ClearRequest => 1,
    }
}

pub(crate) fn recovery_effect_step_count(effect: &RecoveryEffect) -> usize {
    match effect {
        RecoveryEffect::None => 0,
        RecoveryEffect::Rewind { .. } => 1,
        RecoveryEffect::Basebackup { .. } | RecoveryEffect::Bootstrap => 2,
    }
}

pub(crate) fn postgres_effect_step_count(effect: &PostgresEffect) -> usize {
    match effect {
        PostgresEffect::None => 0,
        PostgresEffect::StartPrimary
        | PostgresEffect::StartReplica { .. }
        | PostgresEffect::Promote
        | PostgresEffect::Demote => 1,
    }
}

pub(crate) fn safety_effect_step_count(effect: &SafetyEffect) -> usize {
    match effect {
        SafetyEffect::None => 0,
        SafetyEffect::FenceNode => 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dcs::state::{DcsTrust, MemberRecord, MemberRole, MemberStateClass, PostgresRuntimeClass},
        ha::{
            decision::ReconcileFacts,
            state::{ClusterMode, DesiredNodeState, LeadershipTransferState},
        },
        pginfo::state::{Readiness, SqlStatus},
        process::{
            jobs::ActiveJobKind,
            state::{JobOutcome, ProcessState},
        },
        state::{MemberId, UnixMillis, Version, WorkerStatus},
    };

    use super::{lower_desired_state, HaEffectPlan, PostgresEffect, SafetyEffect};

    fn sample_facts() -> ReconcileFacts {
        ReconcileFacts {
            self_member_id: MemberId("node-2".to_string()),
            cluster_mode: ClusterMode::InitializedLeaderPresent {
                leader: MemberId("node-1".to_string()),
            },
            current_desired_state: DesiredNodeState::Quiescent {
                reason: crate::ha::state::QuiescentReason::WaitingForAuthoritativeLeader,
            },
            leadership_transfer: LeadershipTransferState::None,
            current_process: ProcessState::Idle {
                worker: WorkerStatus::Running,
                last_outcome: None,
            },
            local_member: None,
            trust: DcsTrust::FreshQuorum,
            switchover_pending: false,
            switchover_target: None,
            expected_cluster_identity: None,
            authoritative_leader_member_id: Some(MemberId("node-1".to_string())),
            authoritative_leader_member: None,
            i_am_authoritative_leader: false,
            fresh_unleased_primary_claim_present: false,
            fresh_running_healthy_replica_present: false,
            fresh_running_promotable_replica_present: false,
            replica_targets_authoritative_leader: None,
            postgres_reachable: true,
            postgres_primary: false,
            postgres_replica: false,
            postgres_replay_lsn: None,
            postgres_follow_lsn: None,
            promotion_safety: crate::ha::decision::PromotionSafety { blocker: None },
            elected_candidate: None,
        }
    }

    #[test]
    fn direct_follow_waits_for_replica_metadata_before_demoting() {
        let facts = sample_facts();
        let desired_state = DesiredNodeState::Replica {
            plan: crate::ha::state::ReplicaPlan::Direct {
                leader_member_id: MemberId("node-1".to_string()),
            },
        };

        assert_eq!(
            lower_desired_state(&desired_state, &facts),
            HaEffectPlan::default()
        );
    }

    #[test]
    fn direct_follow_demotes_confirmed_replica_with_wrong_upstream() {
        let mut facts = sample_facts();
        facts.postgres_replica = true;
        facts.replica_targets_authoritative_leader = Some(false);
        let desired_state = DesiredNodeState::Replica {
            plan: crate::ha::state::ReplicaPlan::Direct {
                leader_member_id: MemberId("node-1".to_string()),
            },
        };

        assert_eq!(
            lower_desired_state(&desired_state, &facts),
            HaEffectPlan {
                postgres: PostgresEffect::Demote,
                ..HaEffectPlan::default()
            }
        );
    }

    #[test]
    fn direct_follow_restarts_replica_when_upstream_cannot_be_confirmed() {
        let mut facts = sample_facts();
        facts.postgres_replica = true;
        facts.replica_targets_authoritative_leader = None;
        let desired_state = DesiredNodeState::Replica {
            plan: crate::ha::state::ReplicaPlan::Direct {
                leader_member_id: MemberId("node-1".to_string()),
            },
        };

        assert_eq!(
            lower_desired_state(&desired_state, &facts),
            HaEffectPlan {
                postgres: PostgresEffect::StartReplica {
                    leader_member_id: MemberId("node-1".to_string()),
                },
                ..HaEffectPlan::default()
            }
        );
    }

    #[test]
    fn direct_follow_does_not_demote_primary_while_config_points_to_authoritative_leader() {
        let mut facts = sample_facts();
        facts.postgres_primary = true;
        facts.replica_targets_authoritative_leader = Some(true);
        let desired_state = DesiredNodeState::Replica {
            plan: crate::ha::state::ReplicaPlan::Direct {
                leader_member_id: MemberId("node-1".to_string()),
            },
        };

        assert_eq!(
            lower_desired_state(&desired_state, &facts),
            HaEffectPlan::default()
        );
    }

    #[test]
    fn basebackup_fences_initialized_data_dir_before_reclone() {
        let mut facts = sample_facts();
        facts.postgres_reachable = false;
        facts.local_member = Some(MemberRecord {
            member_id: MemberId("node-2".to_string()),
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            api_url: None,
            role: MemberRole::Replica,
            sql: SqlStatus::Unknown,
            readiness: Readiness::Unknown,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: Some(MemberStateClass::ReplicaOnly),
            postgres_runtime_class: Some(PostgresRuntimeClass::OfflineInspectable),
            updated_at: UnixMillis(1),
            pg_version: Version(1),
        });
        let desired_state = DesiredNodeState::Replica {
            plan: crate::ha::state::ReplicaPlan::Basebackup {
                leader_member_id: MemberId("node-1".to_string()),
            },
        };

        assert_eq!(
            lower_desired_state(&desired_state, &facts),
            HaEffectPlan {
                safety: SafetyEffect::FenceNode,
                ..HaEffectPlan::default()
            }
        );
    }

    #[test]
    fn basebackup_proceeds_after_fencing_succeeds() {
        let mut facts = sample_facts();
        facts.postgres_reachable = false;
        facts.local_member = Some(MemberRecord {
            member_id: MemberId("node-2".to_string()),
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            api_url: None,
            role: MemberRole::Replica,
            sql: SqlStatus::Unknown,
            readiness: Readiness::Unknown,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: Some(MemberStateClass::ReplicaOnly),
            postgres_runtime_class: Some(PostgresRuntimeClass::OfflineInspectable),
            updated_at: UnixMillis(1),
            pg_version: Version(1),
        });
        facts.current_process = ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: Some(JobOutcome::Success {
                id: crate::state::JobId("job-1".to_string()),
                job_kind: ActiveJobKind::Fencing,
                finished_at: UnixMillis(2),
            }),
        };
        let desired_state = DesiredNodeState::Replica {
            plan: crate::ha::state::ReplicaPlan::Basebackup {
                leader_member_id: MemberId("node-1".to_string()),
            },
        };

        assert_eq!(
            lower_desired_state(&desired_state, &facts),
            HaEffectPlan {
                recovery: crate::ha::lower::RecoveryEffect::Basebackup {
                    leader_member_id: MemberId("node-1".to_string()),
                },
                ..HaEffectPlan::default()
            }
        );
    }
}
