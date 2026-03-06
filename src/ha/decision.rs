use serde::{Deserialize, Serialize};

use crate::{
    dcs::state::{DcsTrust, MemberRole},
    pginfo::state::{PgInfoState, SqlStatus},
    process::{
        jobs::ActiveJobKind,
        state::{JobOutcome, ProcessState},
    },
    state::{MemberId, TimelineId},
};

use super::state::{HaPhase, WorldSnapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecisionFacts {
    pub(crate) self_member_id: MemberId,
    pub(crate) trust: DcsTrust,
    pub(crate) postgres_reachable: bool,
    pub(crate) postgres_primary: bool,
    pub(crate) leader_member_id: Option<MemberId>,
    pub(crate) active_leader_member_id: Option<MemberId>,
    pub(crate) available_primary_member_id: Option<MemberId>,
    pub(crate) switchover_requested_by: Option<MemberId>,
    pub(crate) i_am_leader: bool,
    pub(crate) has_other_leader_record: bool,
    pub(crate) has_available_other_leader: bool,
    pub(crate) rewind_required: bool,
    pub(crate) process_state: ProcessState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessActivity {
    Running,
    IdleNoOutcome,
    IdleSuccess,
    IdleFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PhaseOutcome {
    pub(crate) next_phase: HaPhase,
    pub(crate) decision: HaDecision,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum HaDecision {
    #[default]
    NoChange,
    WaitForPostgres {
        start_requested: bool,
        leader_member_id: Option<MemberId>,
    },
    WaitForDcsTrust,
    AttemptLeadership,
    FollowLeader {
        leader_member_id: MemberId,
    },
    BecomePrimary {
        promote: bool,
    },
    StepDown(StepDownPlan),
    RecoverReplica {
        strategy: RecoveryStrategy,
    },
    FenceNode,
    ReleaseLeaderLease {
        reason: LeaseReleaseReason,
    },
    EnterFailSafe {
        release_leader_lease: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StepDownPlan {
    pub(crate) reason: StepDownReason,
    pub(crate) release_leader_lease: bool,
    pub(crate) clear_switchover: bool,
    pub(crate) fence: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum StepDownReason {
    Switchover,
    ForeignLeaderDetected { leader_member_id: MemberId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum RecoveryStrategy {
    Rewind { leader_member_id: MemberId },
    BaseBackup { leader_member_id: MemberId },
    Bootstrap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LeaseReleaseReason {
    FencingComplete,
    PostgresUnreachable,
}

impl DecisionFacts {
    pub(crate) fn from_world(world: &WorldSnapshot) -> Self {
        let self_member_id = MemberId(world.config.value.cluster.member_id.clone());
        let leader_member_id = world
            .dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|record| record.member_id.clone());
        let active_leader_member_id = leader_member_id
            .clone()
            .filter(|leader_id| is_available_primary_leader(world, leader_id));
        let available_primary_member_id = active_leader_member_id.clone().or_else(|| {
            world
                .dcs
                .value
                .cache
                .members
                .values()
                .find(|member| {
                    member.member_id != self_member_id
                        && member.role == MemberRole::Primary
                        && member.sql == SqlStatus::Healthy
                })
                .map(|member| member.member_id.clone())
        });
        let i_am_leader = leader_member_id.as_ref() == Some(&self_member_id);
        let has_other_leader_record = leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);
        let has_available_other_leader = active_leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);

        Self {
            self_member_id,
            trust: world.dcs.value.trust.clone(),
            postgres_reachable: is_postgres_reachable(&world.pg.value),
            postgres_primary: is_local_primary(&world.pg.value),
            leader_member_id,
            active_leader_member_id: active_leader_member_id.clone(),
            available_primary_member_id: available_primary_member_id.clone(),
            switchover_requested_by: world
                .dcs
                .value
                .cache
                .switchover
                .as_ref()
                .map(|request| request.requested_by.clone()),
            i_am_leader,
            has_other_leader_record,
            has_available_other_leader,
            rewind_required: available_primary_member_id
                .as_ref()
                .map(|leader_id| should_rewind_from_leader(world, leader_id))
                .unwrap_or(false),
            process_state: world.process.value.clone(),
        }
    }
}

impl ProcessActivity {
    fn from_process_state(process: &ProcessState, expected_kinds: &[ActiveJobKind]) -> Self {
        match process {
            ProcessState::Running { active, .. } => {
                if expected_kinds.contains(&active.kind) {
                    Self::Running
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { job_kind, .. }),
                ..
            } => {
                if expected_kinds.contains(job_kind) {
                    Self::IdleSuccess
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome:
                    Some(JobOutcome::Failure { job_kind, .. } | JobOutcome::Timeout { job_kind, .. }),
                ..
            } => {
                if expected_kinds.contains(job_kind) {
                    Self::IdleFailure
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome: None, ..
            } => Self::IdleNoOutcome,
        }
    }
}

impl DecisionFacts {
    pub(crate) fn start_postgres_can_be_requested(&self) -> bool {
        !matches!(self.process_state, ProcessState::Running { .. })
    }

    pub(crate) fn rewind_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(&self.process_state, &[ActiveJobKind::PgRewind])
    }

    pub(crate) fn bootstrap_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(
            &self.process_state,
            &[ActiveJobKind::BaseBackup, ActiveJobKind::Bootstrap],
        )
    }

    pub(crate) fn fencing_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(&self.process_state, &[ActiveJobKind::Fencing])
    }
}

impl PhaseOutcome {
    pub(crate) fn new(next_phase: HaPhase, decision: HaDecision) -> Self {
        Self {
            next_phase,
            decision,
        }
    }
}

impl HaDecision {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::NoChange => "no_change",
            Self::WaitForPostgres { .. } => "wait_for_postgres",
            Self::WaitForDcsTrust => "wait_for_dcs_trust",
            Self::AttemptLeadership => "attempt_leadership",
            Self::FollowLeader { .. } => "follow_leader",
            Self::BecomePrimary { .. } => "become_primary",
            Self::StepDown(_) => "step_down",
            Self::RecoverReplica { .. } => "recover_replica",
            Self::FenceNode => "fence_node",
            Self::ReleaseLeaderLease { .. } => "release_leader_lease",
            Self::EnterFailSafe { .. } => "enter_fail_safe",
        }
    }

    pub(crate) fn detail(&self) -> Option<String> {
        match self {
            Self::NoChange | Self::WaitForDcsTrust | Self::AttemptLeadership | Self::FenceNode => {
                None
            }
            Self::WaitForPostgres {
                start_requested,
                leader_member_id,
            } => {
                let leader_detail = leader_member_id
                    .as_ref()
                    .map(|leader| leader.0.as_str())
                    .unwrap_or("none");
                Some(format!(
                    "start_requested={start_requested}, leader_member_id={leader_detail}"
                ))
            }
            Self::FollowLeader { leader_member_id } => Some(leader_member_id.0.clone()),
            Self::BecomePrimary { promote } => Some(format!("promote={promote}")),
            Self::StepDown(plan) => Some(format!(
                "reason={}, release_leader_lease={}, clear_switchover={}, fence={}",
                plan.reason.label(),
                plan.release_leader_lease,
                plan.clear_switchover,
                plan.fence
            )),
            Self::RecoverReplica { strategy } => Some(strategy.label()),
            Self::ReleaseLeaderLease { reason } => Some(reason.label()),
            Self::EnterFailSafe {
                release_leader_lease,
            } => Some(format!("release_leader_lease={release_leader_lease}")),
        }
    }
}

impl StepDownReason {
    fn label(&self) -> String {
        match self {
            Self::Switchover => "switchover".to_string(),
            Self::ForeignLeaderDetected { leader_member_id } => {
                format!("foreign_leader_detected:{}", leader_member_id.0)
            }
        }
    }
}

impl RecoveryStrategy {
    fn label(&self) -> String {
        match self {
            Self::Rewind { leader_member_id } => format!("rewind:{}", leader_member_id.0),
            Self::BaseBackup { leader_member_id } => {
                format!("base_backup:{}", leader_member_id.0)
            }
            Self::Bootstrap => "bootstrap".to_string(),
        }
    }
}

impl LeaseReleaseReason {
    fn label(&self) -> String {
        match self {
            Self::FencingComplete => "fencing_complete".to_string(),
            Self::PostgresUnreachable => "postgres_unreachable".to_string(),
        }
    }
}

fn is_postgres_reachable(state: &PgInfoState) -> bool {
    let sql = match state {
        PgInfoState::Unknown { common } => &common.sql,
        PgInfoState::Primary { common, .. } => &common.sql,
        PgInfoState::Replica { common, .. } => &common.sql,
    };
    matches!(sql, SqlStatus::Healthy)
}

fn is_local_primary(state: &PgInfoState) -> bool {
    matches!(
        state,
        PgInfoState::Primary {
            common,
            ..
        } if matches!(common.sql, SqlStatus::Healthy)
    )
}

fn should_rewind_from_leader(world: &WorldSnapshot, leader_member_id: &MemberId) -> bool {
    let Some(local_timeline) = pg_timeline(&world.pg.value) else {
        return false;
    };

    let leader_timeline = world
        .dcs
        .value
        .cache
        .members
        .get(leader_member_id)
        .and_then(|member| member.timeline);

    leader_timeline
        .map(|timeline| timeline != local_timeline)
        .unwrap_or(false)
}

fn pg_timeline(state: &PgInfoState) -> Option<TimelineId> {
    match state {
        PgInfoState::Unknown { common } => common.timeline,
        PgInfoState::Primary { common, .. } => common.timeline,
        PgInfoState::Replica { common, .. } => common.timeline,
    }
}

fn is_available_primary_leader(world: &WorldSnapshot, leader_member_id: &MemberId) -> bool {
    let leader_record = world.dcs.value.cache.members.get(leader_member_id);

    let Some(member) = leader_record else {
        // Preserve current behavior when leader member metadata is not yet observed.
        return true;
    };

    matches!(member.role, crate::dcs::state::MemberRole::Primary)
        && matches!(member.sql, SqlStatus::Healthy)
}
