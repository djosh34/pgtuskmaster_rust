use super::{
    actions::HaAction,
    decision::{HaDecision, RecoveryStrategy, StepDownPlan},
};

pub(crate) fn lower_decision(decision: &HaDecision) -> Vec<HaAction> {
    match decision {
        HaDecision::NoChange | HaDecision::WaitForDcsTrust => Vec::new(),
        HaDecision::WaitForPostgres { start_requested } => {
            if *start_requested {
                vec![HaAction::StartPostgres]
            } else {
                Vec::new()
            }
        }
        HaDecision::AttemptLeadership => vec![HaAction::AcquireLeaderLease],
        HaDecision::FollowLeader { leader_member_id } => vec![HaAction::FollowLeader {
            leader_member_id: leader_member_id.0.clone(),
        }],
        HaDecision::BecomePrimary { promote } => {
            if *promote {
                vec![HaAction::PromoteToPrimary]
            } else {
                Vec::new()
            }
        }
        HaDecision::StepDown(plan) => lower_step_down(plan),
        HaDecision::RecoverReplica(plan) => match plan.strategy {
            RecoveryStrategy::Rewind => vec![HaAction::StartRewind],
            RecoveryStrategy::BaseBackup => {
                vec![HaAction::WipeDataDir, HaAction::StartBaseBackup]
            }
            RecoveryStrategy::Bootstrap => vec![HaAction::WipeDataDir, HaAction::RunBootstrap],
        },
        HaDecision::FenceNode => vec![HaAction::FenceNode],
        HaDecision::ReleaseLeaderLease { .. } => vec![HaAction::ReleaseLeaderLease],
        HaDecision::EnterFailSafe {
            release_leader_lease,
        } => {
            if *release_leader_lease {
                vec![HaAction::ReleaseLeaderLease, HaAction::SignalFailSafe]
            } else {
                vec![HaAction::SignalFailSafe]
            }
        }
    }
}

fn lower_step_down(plan: &StepDownPlan) -> Vec<HaAction> {
    let demote = vec![HaAction::DemoteToReplica];
    let lease = if plan.release_leader_lease {
        vec![HaAction::ReleaseLeaderLease]
    } else {
        Vec::new()
    };
    let switchover = if plan.clear_switchover {
        vec![HaAction::ClearSwitchover]
    } else {
        Vec::new()
    };
    let fence = if plan.fence {
        vec![HaAction::FenceNode]
    } else {
        Vec::new()
    };

    [demote, lease, switchover, fence].concat()
}

#[cfg(test)]
mod tests {
    use crate::state::MemberId;

    use super::super::decision::{
        HaDecision, LeaseReleaseReason, RecoveryPlan, RecoveryStrategy, StepDownPlan,
        StepDownReason,
    };
    use super::lower_decision;

    #[test]
    fn lowers_composite_step_down_in_order() {
        let decision = HaDecision::StepDown(StepDownPlan {
            reason: StepDownReason::Switchover,
            release_leader_lease: true,
            clear_switchover: true,
            fence: false,
        });

        assert_eq!(
            lower_decision(&decision),
            vec![
                crate::ha::actions::HaAction::DemoteToReplica,
                crate::ha::actions::HaAction::ReleaseLeaderLease,
                crate::ha::actions::HaAction::ClearSwitchover,
            ]
        );
    }

    #[test]
    fn lowers_fail_safe_release_before_signal() {
        let decision = HaDecision::EnterFailSafe {
            release_leader_lease: true,
        };

        assert_eq!(
            lower_decision(&decision),
            vec![
                crate::ha::actions::HaAction::ReleaseLeaderLease,
                crate::ha::actions::HaAction::SignalFailSafe,
            ]
        );
    }

    #[test]
    fn lowers_recovery_variants() {
        let rewind = HaDecision::RecoverReplica(RecoveryPlan {
            strategy: RecoveryStrategy::Rewind,
            leader_member_id: Some(MemberId("node-b".to_string())),
        });
        let basebackup = HaDecision::RecoverReplica(RecoveryPlan {
            strategy: RecoveryStrategy::BaseBackup,
            leader_member_id: Some(MemberId("node-b".to_string())),
        });
        let bootstrap = HaDecision::RecoverReplica(RecoveryPlan {
            strategy: RecoveryStrategy::Bootstrap,
            leader_member_id: None,
        });

        assert_eq!(
            lower_decision(&rewind),
            vec![crate::ha::actions::HaAction::StartRewind]
        );
        assert_eq!(
            lower_decision(&basebackup),
            vec![
                crate::ha::actions::HaAction::WipeDataDir,
                crate::ha::actions::HaAction::StartBaseBackup,
            ]
        );
        assert_eq!(
            lower_decision(&bootstrap),
            vec![
                crate::ha::actions::HaAction::WipeDataDir,
                crate::ha::actions::HaAction::RunBootstrap,
            ]
        );
    }

    #[test]
    fn lowers_extra_release_variant() {
        let decision = HaDecision::ReleaseLeaderLease {
            reason: LeaseReleaseReason::FencingComplete,
        };

        assert_eq!(
            lower_decision(&decision),
            vec![crate::ha::actions::HaAction::ReleaseLeaderLease]
        );
    }
}
