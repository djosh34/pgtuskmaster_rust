use serde::{Deserialize, Serialize};

use crate::{process::jobs::ActiveJobKind, state::MemberId};

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
    Rewind { leader_member_id: MemberId },
    Basebackup { leader_member_id: MemberId },
    Bootstrap,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PostgresEffect {
    #[default]
    None,
    StartPrimary,
    StartReplica { leader_member_id: MemberId },
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
            postgres: match process_activity(&facts.current_process, &[ActiveJobKind::StartPostgres]) {
                ProcessActivity::Running => PostgresEffect::None,
                _ => PostgresEffect::StartPrimary,
            },
            ..HaEffectPlan::default()
        },
    }
}

fn lower_replica(plan: &ReplicaPlan, facts: &ReconcileFacts) -> HaEffectPlan {
    match plan {
        ReplicaPlan::DirectFollow { leader_member_id } => lower_direct_follow(leader_member_id, facts),
        ReplicaPlan::RewindThenFollow { leader_member_id } => lower_rewind_then_follow(leader_member_id, facts),
        ReplicaPlan::BasebackupThenFollow { leader_member_id } => {
            lower_basebackup_then_follow(leader_member_id, facts)
        }
    }
}

fn lower_direct_follow(leader_member_id: &MemberId, facts: &ReconcileFacts) -> HaEffectPlan {
    if facts.postgres_primary {
        return HaEffectPlan {
            postgres: PostgresEffect::Demote,
            ..HaEffectPlan::default()
        };
    }

    if facts.postgres_reachable {
        return maybe_clear_switchover(facts, HaEffectPlan::default());
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
