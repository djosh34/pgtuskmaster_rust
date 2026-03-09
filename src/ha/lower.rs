use serde::{Deserialize, Serialize};

use crate::state::MemberId;

use super::decision::{HaDecision, RecoveryStrategy, StepDownPlan};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HaEffectPlan {
    pub(crate) lease: LeaseEffect,
    pub(crate) switchover: SwitchoverEffect,
    pub(crate) replication: ReplicationEffect,
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
pub(crate) enum ReplicationEffect {
    #[default]
    None,
    FollowLeader {
        leader_member_id: MemberId,
    },
    RecoverReplica {
        strategy: RecoveryStrategy,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PostgresEffect {
    #[default]
    None,
    Start,
    Promote,
    Demote,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SafetyEffect {
    #[default]
    None,
    FenceNode,
    SignalFailSafe,
}

impl HaDecision {
    pub(crate) fn lower(&self) -> HaEffectPlan {
        match self {
            Self::NoChange | Self::WaitForDcsTrust => HaEffectPlan::default(),
            Self::WaitForPostgres {
                start_requested, ..
            } => HaEffectPlan {
                postgres: if *start_requested {
                    PostgresEffect::Start
                } else {
                    PostgresEffect::None
                },
                ..HaEffectPlan::default()
            },
            Self::AttemptLeadership => HaEffectPlan {
                lease: LeaseEffect::AcquireLeader,
                ..HaEffectPlan::default()
            },
            Self::FollowLeader { leader_member_id } => HaEffectPlan {
                replication: ReplicationEffect::FollowLeader {
                    leader_member_id: leader_member_id.clone(),
                },
                ..HaEffectPlan::default()
            },
            Self::BecomePrimary { promote } => HaEffectPlan {
                postgres: if *promote {
                    PostgresEffect::Promote
                } else {
                    PostgresEffect::None
                },
                ..HaEffectPlan::default()
            },
            Self::CompleteSwitchover => HaEffectPlan {
                switchover: SwitchoverEffect::ClearRequest,
                ..HaEffectPlan::default()
            },
            Self::StepDown(plan) => lower_step_down(plan),
            Self::RecoverReplica { strategy } => HaEffectPlan {
                replication: ReplicationEffect::RecoverReplica {
                    strategy: strategy.clone(),
                },
                ..HaEffectPlan::default()
            },
            Self::FenceNode => HaEffectPlan {
                safety: SafetyEffect::FenceNode,
                ..HaEffectPlan::default()
            },
            Self::ReleaseLeaderLease { .. } => HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                ..HaEffectPlan::default()
            },
            Self::EnterFailSafe {
                release_leader_lease,
            } => HaEffectPlan {
                lease: if *release_leader_lease {
                    LeaseEffect::ReleaseLeader
                } else {
                    LeaseEffect::None
                },
                safety: SafetyEffect::FenceNode,
                ..HaEffectPlan::default()
            },
        }
    }
}

pub(crate) fn lower_decision(decision: &HaDecision) -> HaEffectPlan {
    decision.lower()
}

impl HaEffectPlan {
    pub(crate) fn len(&self) -> usize {
        self.dispatch_step_count()
    }

    pub(crate) fn dispatch_step_count(&self) -> usize {
        lease_effect_step_count(&self.lease)
            + switchover_effect_step_count(&self.switchover)
            + replication_effect_step_count(&self.replication)
            + postgres_effect_step_count(&self.postgres)
            + safety_effect_step_count(&self.safety)
    }
}

fn lower_step_down(plan: &StepDownPlan) -> HaEffectPlan {
    HaEffectPlan {
        lease: if plan.release_leader_lease {
            LeaseEffect::ReleaseLeader
        } else {
            LeaseEffect::None
        },
        switchover: SwitchoverEffect::None,
        replication: ReplicationEffect::None,
        postgres: PostgresEffect::Demote,
        safety: if plan.fence {
            SafetyEffect::FenceNode
        } else {
            SafetyEffect::None
        },
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

pub(crate) fn replication_effect_step_count(effect: &ReplicationEffect) -> usize {
    match effect {
        ReplicationEffect::None => 0,
        ReplicationEffect::FollowLeader { .. } => 1,
        ReplicationEffect::RecoverReplica { strategy } => match strategy {
            RecoveryStrategy::Rewind { .. } => 1,
            RecoveryStrategy::BaseBackup { .. } | RecoveryStrategy::Bootstrap => 2,
        },
    }
}

pub(crate) fn postgres_effect_step_count(effect: &PostgresEffect) -> usize {
    match effect {
        PostgresEffect::None => 0,
        PostgresEffect::Start | PostgresEffect::Promote | PostgresEffect::Demote => 1,
    }
}

pub(crate) fn safety_effect_step_count(effect: &SafetyEffect) -> usize {
    match effect {
        SafetyEffect::None => 0,
        SafetyEffect::FenceNode | SafetyEffect::SignalFailSafe => 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::state::MemberId;

    use super::{
        super::decision::{
            HaDecision, LeaseReleaseReason, RecoveryStrategy, StepDownPlan, StepDownReason,
        },
        HaEffectPlan, LeaseEffect, PostgresEffect, ReplicationEffect, SafetyEffect,
        SwitchoverEffect,
    };

    #[test]
    fn lowers_composite_step_down_into_bucketed_plan() {
        let decision = HaDecision::StepDown(StepDownPlan {
            reason: StepDownReason::Switchover,
            release_leader_lease: true,
            fence: false,
        });

        assert_eq!(
            decision.lower(),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Demote,
                safety: SafetyEffect::None,
            }
        );
    }

    #[test]
    fn lowers_complete_switchover_into_clear_request_plan() {
        let decision = HaDecision::CompleteSwitchover;

        assert_eq!(
            decision.lower(),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::ClearRequest,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
    }

    #[test]
    fn lowers_fail_safe_primary_release_into_fencing_plan() {
        let decision = HaDecision::EnterFailSafe {
            release_leader_lease: true,
        };

        assert_eq!(
            decision.lower(),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::FenceNode,
            }
        );
    }

    #[test]
    fn lowers_fail_safe_without_release_into_fencing_plan() {
        let decision = HaDecision::EnterFailSafe {
            release_leader_lease: false,
        };

        assert_eq!(
            decision.lower(),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::FenceNode,
            }
        );
    }

    #[test]
    fn lowers_recovery_variants() {
        let rewind = HaDecision::RecoverReplica {
            strategy: RecoveryStrategy::Rewind {
                leader_member_id: MemberId("node-b".to_string()),
            },
        };
        let basebackup = HaDecision::RecoverReplica {
            strategy: RecoveryStrategy::BaseBackup {
                leader_member_id: MemberId("node-b".to_string()),
            },
        };
        let bootstrap = HaDecision::RecoverReplica {
            strategy: RecoveryStrategy::Bootstrap,
        };

        assert_eq!(
            rewind.lower(),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::RecoverReplica {
                    strategy: RecoveryStrategy::Rewind {
                        leader_member_id: MemberId("node-b".to_string()),
                    },
                },
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
        assert_eq!(basebackup.lower().dispatch_step_count(), 2);
        assert_eq!(bootstrap.lower().dispatch_step_count(), 2);
    }

    #[test]
    fn lowers_extra_release_variant() {
        let decision = HaDecision::ReleaseLeaderLease {
            reason: LeaseReleaseReason::FencingComplete,
        };

        assert_eq!(
            decision.lower(),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
    }
}
