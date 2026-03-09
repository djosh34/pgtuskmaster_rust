use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    dcs::state::{member_record_is_fresh, DcsTrust, MemberRecord, MemberRole},
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    process::{
        jobs::ActiveJobKind,
        state::{JobOutcome, ProcessState},
    },
    state::{MemberId, TimelineId, WalLsn},
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
    pub(crate) followable_member_id: Option<MemberId>,
    pub(crate) switchover_pending: bool,
    pub(crate) pending_switchover_target: Option<MemberId>,
    pub(crate) eligible_switchover_targets: BTreeSet<MemberId>,
    pub(crate) i_am_leader: bool,
    pub(crate) has_other_leader_record: bool,
    pub(crate) has_available_other_leader: bool,
    pub(crate) rewind_required: bool,
    pub(crate) promotion_safety: PromotionSafety,
    pub(crate) process_state: ProcessState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PromotionSafety {
    pub(crate) blocker: Option<PromotionSafetyBlocker>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PromotionSafetyBlocker {
    NotHealthyReplica,
    MissingLocalTimeline,
    MissingLocalReplayLsn,
    HigherFreshTimeline {
        required_timeline: TimelineId,
        source_member_id: MemberId,
    },
    LaggingFreshWal {
        timeline: TimelineId,
        required_lsn: WalLsn,
        local_replay_lsn: WalLsn,
        source_member_id: MemberId,
    },
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
    WaitForPromotionSafety {
        blocker: PromotionSafetyBlocker,
    },
    AttemptLeadership,
    FollowLeader {
        leader_member_id: MemberId,
    },
    BecomePrimary {
        promote: bool,
    },
    CompleteSwitchover,
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
        let followable_member_id = active_leader_member_id.clone().or_else(|| {
            world
                .dcs
                .value
                .cache
                .members
                .values()
                .find(|member| {
                    member.member_id != self_member_id
                        && member_record_is_fresh(
                            member,
                            &world.dcs.value.cache,
                            world.dcs.updated_at,
                        )
                        && member.role == MemberRole::Primary
                        && member.sql == SqlStatus::Healthy
                        && member.readiness == Readiness::Ready
                })
                .map(|member| member.member_id.clone())
        });
        let eligible_switchover_targets = eligible_switchover_targets(world);
        let i_am_leader = leader_member_id.as_ref() == Some(&self_member_id);
        let has_other_leader_record = leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);
        let has_available_other_leader = active_leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);
        let promotion_safety = evaluate_promotion_safety(
            &local_promotion_candidate(world, &self_member_id),
            &world.dcs.value.cache,
            world.dcs.updated_at,
        );

        Self {
            self_member_id,
            trust: world.dcs.value.trust.clone(),
            postgres_reachable: is_postgres_reachable(&world.pg.value),
            postgres_primary: is_local_primary(&world.pg.value),
            leader_member_id,
            active_leader_member_id: active_leader_member_id.clone(),
            followable_member_id: followable_member_id.clone(),
            switchover_pending: world.dcs.value.cache.switchover.is_some(),
            pending_switchover_target: world
                .dcs
                .value
                .cache
                .switchover
                .as_ref()
                .and_then(|request| request.switchover_to.clone()),
            eligible_switchover_targets,
            i_am_leader,
            has_other_leader_record,
            has_available_other_leader,
            rewind_required: followable_member_id
                .as_ref()
                .map(|leader_id| should_rewind_from_leader(world, leader_id))
                .unwrap_or(false),
            promotion_safety,
            process_state: world.process.value.clone(),
        }
    }
}

impl PromotionSafety {
    fn safe() -> Self {
        Self { blocker: None }
    }

    fn blocked(blocker: PromotionSafetyBlocker) -> Self {
        Self {
            blocker: Some(blocker),
        }
    }

    pub(crate) fn allows_promotion(&self) -> bool {
        self.blocker.is_none()
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

    pub(crate) fn switchover_target_is_eligible(&self, member_id: &MemberId) -> bool {
        self.eligible_switchover_targets.contains(member_id)
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
            Self::WaitForPromotionSafety { .. } => "wait_for_promotion_safety",
            Self::AttemptLeadership => "attempt_leadership",
            Self::FollowLeader { .. } => "follow_leader",
            Self::BecomePrimary { .. } => "become_primary",
            Self::CompleteSwitchover => "complete_switchover",
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
            Self::WaitForPromotionSafety { blocker } => Some(blocker.detail()),
            Self::FollowLeader { leader_member_id } => Some(leader_member_id.0.clone()),
            Self::BecomePrimary { promote } => Some(format!("promote={promote}")),
            Self::CompleteSwitchover => None,
            Self::StepDown(plan) => Some(format!(
                "reason={}, release_leader_lease={}, fence={}",
                plan.reason.label(),
                plan.release_leader_lease,
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

impl PromotionSafetyBlocker {
    fn detail(&self) -> String {
        match self {
            Self::NotHealthyReplica => "not_healthy_replica".to_string(),
            Self::MissingLocalTimeline => "missing_local_timeline".to_string(),
            Self::MissingLocalReplayLsn => "missing_local_replay_lsn".to_string(),
            Self::HigherFreshTimeline {
                required_timeline,
                source_member_id,
            } => format!(
                "higher_fresh_timeline(required_timeline={}, source_member_id={})",
                required_timeline.0, source_member_id.0
            ),
            Self::LaggingFreshWal {
                timeline,
                required_lsn,
                local_replay_lsn,
                source_member_id,
            } => format!(
                "lagging_fresh_wal(timeline={}, required_lsn={}, local_replay_lsn={}, source_member_id={})",
                timeline.0, required_lsn.0, local_replay_lsn.0, source_member_id.0
            ),
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
    if !is_local_primary(&world.pg.value) {
        return false;
    }

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
    let Some(member) = world.dcs.value.cache.members.get(leader_member_id) else {
        return false;
    };

    member_record_is_fresh(member, &world.dcs.value.cache, world.dcs.updated_at)
        && matches!(member.role, crate::dcs::state::MemberRole::Primary)
        && matches!(member.sql, SqlStatus::Healthy)
        && matches!(member.readiness, Readiness::Ready)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PromotionCandidate {
    member_id: MemberId,
    role: MemberRole,
    sql: SqlStatus,
    readiness: Readiness,
    timeline: Option<TimelineId>,
    replay_lsn: Option<WalLsn>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FreshWalEvidence {
    member_id: MemberId,
    timeline: TimelineId,
    wal_lsn: WalLsn,
}

fn local_promotion_candidate(
    world: &WorldSnapshot,
    self_member_id: &MemberId,
) -> PromotionCandidate {
    match &world.pg.value {
        PgInfoState::Unknown { common } => PromotionCandidate {
            member_id: self_member_id.clone(),
            role: MemberRole::Unknown,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            replay_lsn: None,
        },
        PgInfoState::Primary { common, .. } => PromotionCandidate {
            member_id: self_member_id.clone(),
            role: MemberRole::Primary,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            replay_lsn: None,
        },
        PgInfoState::Replica {
            common, replay_lsn, ..
        } => PromotionCandidate {
            member_id: self_member_id.clone(),
            role: MemberRole::Replica,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            replay_lsn: Some(*replay_lsn),
        },
    }
}

fn member_promotion_candidate(member: &MemberRecord) -> PromotionCandidate {
    PromotionCandidate {
        member_id: member.member_id.clone(),
        role: member.role.clone(),
        sql: member.sql.clone(),
        readiness: member.readiness.clone(),
        timeline: member.timeline,
        replay_lsn: member.replay_lsn,
    }
}

fn member_fresh_wal_evidence(
    member: &MemberRecord,
    cache: &crate::dcs::state::DcsCache,
    observed_at: crate::state::UnixMillis,
) -> Option<FreshWalEvidence> {
    if !member_record_is_fresh(member, cache, observed_at) {
        return None;
    }

    let timeline = member.timeline?;
    let wal_lsn = member.write_lsn.or(member.replay_lsn)?;

    Some(FreshWalEvidence {
        member_id: member.member_id.clone(),
        timeline,
        wal_lsn,
    })
}

fn evaluate_promotion_safety(
    candidate: &PromotionCandidate,
    cache: &crate::dcs::state::DcsCache,
    observed_at: crate::state::UnixMillis,
) -> PromotionSafety {
    if candidate.role != MemberRole::Replica
        || candidate.sql != SqlStatus::Healthy
        || candidate.readiness != Readiness::Ready
    {
        return PromotionSafety::blocked(PromotionSafetyBlocker::NotHealthyReplica);
    }

    let Some(local_timeline) = candidate.timeline else {
        return PromotionSafety::blocked(PromotionSafetyBlocker::MissingLocalTimeline);
    };
    let Some(local_replay_lsn) = candidate.replay_lsn else {
        return PromotionSafety::blocked(PromotionSafetyBlocker::MissingLocalReplayLsn);
    };

    let highest_timeline = cache
        .members
        .values()
        .filter(|member| member_record_is_fresh(member, cache, observed_at))
        .filter_map(|member| {
            member
                .timeline
                .map(|timeline| (timeline, member.member_id.clone()))
        })
        .max_by_key(|(timeline, _)| *timeline);

    if let Some((required_timeline, source_member_id)) = highest_timeline {
        if local_timeline < required_timeline {
            return PromotionSafety::blocked(PromotionSafetyBlocker::HigherFreshTimeline {
                required_timeline,
                source_member_id,
            });
        }
    }

    let highest_wal = cache
        .members
        .values()
        .filter_map(|member| member_fresh_wal_evidence(member, cache, observed_at))
        .filter(|evidence| evidence.timeline == local_timeline)
        .max_by_key(|evidence| evidence.wal_lsn);

    if let Some(required) = highest_wal {
        if local_replay_lsn < required.wal_lsn {
            return PromotionSafety::blocked(PromotionSafetyBlocker::LaggingFreshWal {
                timeline: local_timeline,
                required_lsn: required.wal_lsn,
                local_replay_lsn,
                source_member_id: required.member_id,
            });
        }
    }

    PromotionSafety::safe()
}

pub(crate) fn eligible_switchover_targets(world: &WorldSnapshot) -> BTreeSet<MemberId> {
    world
        .dcs
        .value
        .cache
        .members
        .values()
        .filter(|member| {
            switchover_target_is_eligible_member(
                member,
                &world.dcs.value.cache,
                world.dcs.updated_at,
            )
        })
        .map(|member| member.member_id.clone())
        .collect()
}

pub(crate) fn switchover_target_is_eligible_member(
    member: &MemberRecord,
    cache: &crate::dcs::state::DcsCache,
    observed_at: crate::state::UnixMillis,
) -> bool {
    member_record_is_fresh(member, cache, observed_at)
        && evaluate_promotion_safety(&member_promotion_candidate(member), cache, observed_at)
            .allows_promotion()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::RuntimeConfig,
        dcs::state::{DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{MemberId, TimelineId, UnixMillis, Version, Versioned, WalLsn, WorkerStatus},
    };

    use super::{eligible_switchover_targets, DecisionFacts, PromotionSafetyBlocker};
    use crate::ha::state::WorldSnapshot;

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_member(
        member_id: &str,
        role: MemberRole,
        sql: SqlStatus,
        readiness: Readiness,
        updated_at: UnixMillis,
    ) -> MemberRecord {
        MemberRecord {
            member_id: MemberId(member_id.to_string()),
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            api_url: None,
            role,
            sql,
            readiness,
            timeline: Some(TimelineId(1)),
            write_lsn: None,
            replay_lsn: None,
            updated_at,
            pg_version: Version(1),
        }
    }

    fn replica_member_with_replay_lsn(
        member_id: &str,
        timeline: TimelineId,
        replay_lsn: u64,
    ) -> MemberRecord {
        let mut member = sample_member(
            member_id,
            MemberRole::Replica,
            SqlStatus::Healthy,
            Readiness::Ready,
            UnixMillis(100),
        );
        member.timeline = Some(timeline);
        member.replay_lsn = Some(WalLsn(replay_lsn));
        member
    }

    fn primary_member_with_write_lsn(
        member_id: &str,
        timeline: TimelineId,
        write_lsn: u64,
    ) -> MemberRecord {
        let mut member = sample_member(
            member_id,
            MemberRole::Primary,
            SqlStatus::Healthy,
            Readiness::Ready,
            UnixMillis(100),
        );
        member.timeline = Some(timeline);
        member.write_lsn = Some(WalLsn(write_lsn));
        member
    }

    fn sample_world(cache: DcsCache, trust: DcsTrust, now: UnixMillis) -> WorldSnapshot {
        let config = cache.config.clone();
        WorldSnapshot {
            config: Versioned::new(Version(1), now, config),
            pg: Versioned::new(
                Version(1),
                now,
                PgInfoState::Replica {
                    common: PgInfoCommon {
                        worker: WorkerStatus::Running,
                        sql: SqlStatus::Healthy,
                        readiness: Readiness::Ready,
                        timeline: Some(TimelineId(1)),
                        pg_config: PgConfig {
                            port: None,
                            hot_standby: None,
                            primary_conninfo: None,
                            primary_slot_name: None,
                            extra: BTreeMap::new(),
                        },
                        last_refresh_at: Some(now),
                    },
                    replay_lsn: crate::state::WalLsn(10),
                    follow_lsn: None,
                    upstream: None,
                },
            ),
            dcs: Versioned::new(
                Version(1),
                now,
                DcsState {
                    worker: WorkerStatus::Running,
                    trust,
                    cache,
                    last_refresh_at: Some(now),
                },
            ),
            process: Versioned::new(
                Version(1),
                now,
                ProcessState::Idle {
                    worker: WorkerStatus::Running,
                    last_outcome: None,
                },
            ),
        }
    }

    #[test]
    fn eligible_switchover_targets_exclude_stale_replicas() {
        let mut cache = DcsCache {
            members: BTreeMap::new(),
            leader: Some(LeaderRecord {
                member_id: MemberId("node-a".to_string()),
            }),
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        cache.members.insert(
            MemberId("node-a".to_string()),
            sample_member(
                "node-a",
                MemberRole::Primary,
                SqlStatus::Healthy,
                Readiness::Ready,
                UnixMillis(100),
            ),
        );
        cache.members.insert(
            MemberId("node-b".to_string()),
            sample_member(
                "node-b",
                MemberRole::Replica,
                SqlStatus::Healthy,
                Readiness::Ready,
                UnixMillis(1),
            ),
        );

        let world = sample_world(cache, DcsTrust::FullQuorum, UnixMillis(20_000));
        let eligible = eligible_switchover_targets(&world);

        assert!(eligible.is_empty());
    }

    #[test]
    fn eligible_switchover_targets_require_catch_up_to_primary_write_lsn() {
        let mut cache = DcsCache {
            members: BTreeMap::new(),
            leader: Some(LeaderRecord {
                member_id: MemberId("node-a".to_string()),
            }),
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        cache.members.insert(
            MemberId("node-a".to_string()),
            primary_member_with_write_lsn("node-a", TimelineId(1), 25),
        );
        cache.members.insert(
            MemberId("node-b".to_string()),
            replica_member_with_replay_lsn("node-b", TimelineId(1), 10),
        );
        cache.members.insert(
            MemberId("node-c".to_string()),
            replica_member_with_replay_lsn("node-c", TimelineId(1), 25),
        );

        let eligible = eligible_switchover_targets(&sample_world(
            cache,
            DcsTrust::FullQuorum,
            UnixMillis(100),
        ));

        assert!(!eligible.contains(&MemberId("node-b".to_string())));
        assert!(eligible.contains(&MemberId("node-c".to_string())));
    }

    #[test]
    fn decision_facts_drop_active_leader_when_member_metadata_is_missing() {
        let mut cache = DcsCache {
            members: BTreeMap::new(),
            leader: Some(LeaderRecord {
                member_id: MemberId("node-b".to_string()),
            }),
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        cache.members.insert(
            MemberId("node-a".to_string()),
            sample_member(
                "node-a",
                MemberRole::Replica,
                SqlStatus::Healthy,
                Readiness::Ready,
                UnixMillis(100),
            ),
        );

        let facts =
            DecisionFacts::from_world(&sample_world(cache, DcsTrust::FullQuorum, UnixMillis(100)));

        assert_eq!(facts.leader_member_id, Some(MemberId("node-b".to_string())));
        assert_eq!(facts.active_leader_member_id, None);
        assert!(!facts.has_available_other_leader);
    }

    #[test]
    fn decision_facts_drop_active_leader_when_member_is_stale() {
        let mut cache = DcsCache {
            members: BTreeMap::new(),
            leader: Some(LeaderRecord {
                member_id: MemberId("node-b".to_string()),
            }),
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        cache.members.insert(
            MemberId("node-a".to_string()),
            sample_member(
                "node-a",
                MemberRole::Replica,
                SqlStatus::Healthy,
                Readiness::Ready,
                UnixMillis(100),
            ),
        );
        cache.members.insert(
            MemberId("node-b".to_string()),
            sample_member(
                "node-b",
                MemberRole::Primary,
                SqlStatus::Healthy,
                Readiness::Ready,
                UnixMillis(1),
            ),
        );

        let facts = DecisionFacts::from_world(&sample_world(
            cache,
            DcsTrust::FullQuorum,
            UnixMillis(20_000),
        ));

        assert_eq!(facts.active_leader_member_id, None);
        assert_eq!(facts.followable_member_id, None);
    }

    #[test]
    fn decision_facts_block_promotion_when_replica_lags_fresh_replica() {
        let mut cache = DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        cache.members.insert(
            MemberId("node-a".to_string()),
            replica_member_with_replay_lsn("node-a", TimelineId(1), 10),
        );
        cache.members.insert(
            MemberId("node-b".to_string()),
            replica_member_with_replay_lsn("node-b", TimelineId(1), 20),
        );

        let facts =
            DecisionFacts::from_world(&sample_world(cache, DcsTrust::FullQuorum, UnixMillis(100)));

        assert_eq!(
            facts.promotion_safety.blocker,
            Some(PromotionSafetyBlocker::LaggingFreshWal {
                timeline: TimelineId(1),
                required_lsn: WalLsn(20),
                local_replay_lsn: WalLsn(10),
                source_member_id: MemberId("node-b".to_string()),
            })
        );
    }

    #[test]
    fn decision_facts_block_promotion_when_higher_timeline_is_fresh() {
        let mut cache = DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        cache.members.insert(
            MemberId("node-a".to_string()),
            replica_member_with_replay_lsn("node-a", TimelineId(1), 20),
        );
        cache.members.insert(
            MemberId("node-b".to_string()),
            replica_member_with_replay_lsn("node-b", TimelineId(2), 1),
        );

        let facts =
            DecisionFacts::from_world(&sample_world(cache, DcsTrust::FullQuorum, UnixMillis(100)));

        assert_eq!(
            facts.promotion_safety.blocker,
            Some(PromotionSafetyBlocker::HigherFreshTimeline {
                required_timeline: TimelineId(2),
                source_member_id: MemberId("node-b".to_string()),
            })
        );
    }

    #[test]
    fn decision_facts_block_promotion_when_local_timeline_is_unknown() {
        let cache = DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        let config = cache.config.clone();
        let now = UnixMillis(100);
        let facts = DecisionFacts::from_world(&WorldSnapshot {
            config: Versioned::new(Version(1), now, config.clone()),
            pg: Versioned::new(
                Version(1),
                now,
                PgInfoState::Replica {
                    common: PgInfoCommon {
                        worker: WorkerStatus::Running,
                        sql: SqlStatus::Healthy,
                        readiness: Readiness::Ready,
                        timeline: None,
                        pg_config: PgConfig {
                            port: None,
                            hot_standby: None,
                            primary_conninfo: None,
                            primary_slot_name: None,
                            extra: BTreeMap::new(),
                        },
                        last_refresh_at: Some(now),
                    },
                    replay_lsn: WalLsn(10),
                    follow_lsn: None,
                    upstream: None,
                },
            ),
            dcs: Versioned::new(
                Version(1),
                now,
                DcsState {
                    worker: WorkerStatus::Running,
                    trust: DcsTrust::FullQuorum,
                    cache,
                    last_refresh_at: Some(now),
                },
            ),
            process: Versioned::new(
                Version(1),
                now,
                ProcessState::Idle {
                    worker: WorkerStatus::Running,
                    last_outcome: None,
                },
            ),
        });

        assert_eq!(
            facts.promotion_safety.blocker,
            Some(PromotionSafetyBlocker::MissingLocalTimeline)
        );
    }

    #[test]
    fn decision_facts_allow_promotion_when_replica_is_caught_up() {
        let mut cache = DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        cache.members.insert(
            MemberId("node-a".to_string()),
            replica_member_with_replay_lsn("node-a", TimelineId(1), 20),
        );
        cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member_with_write_lsn("node-b", TimelineId(1), 20),
        );
        let config = cache.config.clone();
        let now = UnixMillis(100);
        let facts = DecisionFacts::from_world(&WorldSnapshot {
            config: Versioned::new(Version(1), now, config.clone()),
            pg: Versioned::new(
                Version(1),
                now,
                PgInfoState::Replica {
                    common: PgInfoCommon {
                        worker: WorkerStatus::Running,
                        sql: SqlStatus::Healthy,
                        readiness: Readiness::Ready,
                        timeline: Some(TimelineId(1)),
                        pg_config: PgConfig {
                            port: None,
                            hot_standby: None,
                            primary_conninfo: None,
                            primary_slot_name: None,
                            extra: BTreeMap::new(),
                        },
                        last_refresh_at: Some(now),
                    },
                    replay_lsn: WalLsn(20),
                    follow_lsn: None,
                    upstream: None,
                },
            ),
            dcs: Versioned::new(
                Version(1),
                now,
                DcsState {
                    worker: WorkerStatus::Running,
                    trust: DcsTrust::FullQuorum,
                    cache,
                    last_refresh_at: Some(now),
                },
            ),
            process: Versioned::new(
                Version(1),
                now,
                ProcessState::Idle {
                    worker: WorkerStatus::Running,
                    last_outcome: None,
                },
            ),
        });

        assert!(facts.promotion_safety.allows_promotion());
    }

    #[test]
    fn decision_facts_do_not_require_rewind_for_replica_timeline_mismatch() {
        let mut cache = DcsCache {
            members: BTreeMap::new(),
            leader: Some(LeaderRecord {
                member_id: MemberId("node-b".to_string()),
            }),
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        cache.members.insert(
            MemberId("node-a".to_string()),
            sample_member(
                "node-a",
                MemberRole::Replica,
                SqlStatus::Healthy,
                Readiness::Ready,
                UnixMillis(100),
            ),
        );
        let mut leader = sample_member(
            "node-b",
            MemberRole::Primary,
            SqlStatus::Healthy,
            Readiness::Ready,
            UnixMillis(100),
        );
        leader.timeline = Some(TimelineId(2));
        cache.members.insert(MemberId("node-b".to_string()), leader);

        let facts =
            DecisionFacts::from_world(&sample_world(cache, DcsTrust::FullQuorum, UnixMillis(100)));

        assert_eq!(
            facts.active_leader_member_id,
            Some(MemberId("node-b".to_string()))
        );
        assert_eq!(
            facts.followable_member_id,
            Some(MemberId("node-b".to_string()))
        );
        assert!(!facts.rewind_required);
    }

    #[test]
    fn decision_facts_require_rewind_for_primary_timeline_mismatch() {
        let mut cache = DcsCache {
            members: BTreeMap::new(),
            leader: Some(LeaderRecord {
                member_id: MemberId("node-b".to_string()),
            }),
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        };
        cache.members.insert(
            MemberId("node-a".to_string()),
            sample_member(
                "node-a",
                MemberRole::Primary,
                SqlStatus::Healthy,
                Readiness::Ready,
                UnixMillis(100),
            ),
        );
        let mut leader = sample_member(
            "node-b",
            MemberRole::Primary,
            SqlStatus::Healthy,
            Readiness::Ready,
            UnixMillis(100),
        );
        leader.timeline = Some(TimelineId(2));
        cache.members.insert(MemberId("node-b".to_string()), leader);

        let config = cache.config.clone();
        let now = UnixMillis(100);
        let facts = DecisionFacts::from_world(&WorldSnapshot {
            config: Versioned::new(Version(1), now, config.clone()),
            pg: Versioned::new(
                Version(1),
                now,
                PgInfoState::Primary {
                    common: PgInfoCommon {
                        worker: WorkerStatus::Running,
                        sql: SqlStatus::Healthy,
                        readiness: Readiness::Ready,
                        timeline: Some(TimelineId(1)),
                        pg_config: PgConfig {
                            port: None,
                            hot_standby: None,
                            primary_conninfo: None,
                            primary_slot_name: None,
                            extra: BTreeMap::new(),
                        },
                        last_refresh_at: Some(now),
                    },
                    wal_lsn: crate::state::WalLsn(10),
                    slots: vec![],
                },
            ),
            dcs: Versioned::new(
                Version(1),
                now,
                DcsState {
                    worker: WorkerStatus::Running,
                    trust: DcsTrust::FullQuorum,
                    cache,
                    last_refresh_at: Some(now),
                },
            ),
            process: Versioned::new(
                Version(1),
                now,
                ProcessState::Idle {
                    worker: WorkerStatus::Running,
                    last_outcome: None,
                },
            ),
        });

        assert_eq!(
            facts.active_leader_member_id,
            Some(MemberId("node-b".to_string()))
        );
        assert_eq!(
            facts.followable_member_id,
            Some(MemberId("node-b".to_string()))
        );
        assert!(facts.rewind_required);
    }
}
