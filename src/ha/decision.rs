use std::{cmp::Ordering, collections::BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    dcs::state::{
        member_record_is_fresh, DcsTrust, DcsView, MemberRecord, MemberRole, MemberStateClass,
        PostgresRuntimeClass,
    },
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    process::{
        jobs::ActiveJobKind,
        state::{JobOutcome, ProcessState},
    },
    state::{MemberId, SystemIdentifier, TimelineId, UnixMillis, WalLsn},
};

use super::state::{
    BootstrapPlan, ClusterMode, DesiredNodeState, FencePlan, HaState, LeadershipTransferState,
    PrimaryPlan, QuiescentReason, ReplicaPlan, WorldSnapshot,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReconcileFacts {
    pub(crate) self_member_id: MemberId,
    pub(crate) cluster_mode: ClusterMode,
    pub(crate) leadership_transfer: LeadershipTransferState,
    pub(crate) current_process: ProcessState,
    pub(crate) local_member: Option<MemberRecord>,
    pub(crate) trust: DcsTrust,
    pub(crate) switchover_pending: bool,
    pub(crate) switchover_target: Option<MemberId>,
    pub(crate) expected_cluster_identity: Option<SystemIdentifier>,
    pub(crate) authoritative_leader_member_id: Option<MemberId>,
    pub(crate) authoritative_leader_member: Option<MemberRecord>,
    pub(crate) i_am_authoritative_leader: bool,
    pub(crate) postgres_reachable: bool,
    pub(crate) postgres_primary: bool,
    pub(crate) postgres_replica: bool,
    pub(crate) promotion_safety: PromotionSafety,
    pub(crate) elected_candidate: Option<MemberId>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct CandidateDescriptor {
    member: MemberRecord,
    identity_rank: u8,
    eligible: bool,
    timeline_rank: u64,
    lsn_rank: u64,
    runtime_rank: u8,
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

pub(crate) fn reconcile_facts(current: &HaState, world: &WorldSnapshot) -> ReconcileFacts {
    let self_member_id = MemberId(world.config.value.cluster.member_id.clone());
    let cluster_mode = derive_cluster_mode(&world.dcs.value, &self_member_id);
    let authoritative_leader_member_id = match &cluster_mode {
        ClusterMode::InitializedLeaderPresent { leader } => Some(leader.clone()),
        _ => None,
    };
    let authoritative_leader_member = authoritative_leader_member_id
        .as_ref()
        .and_then(|leader_id| world.dcs.value.cache.members.get(leader_id))
        .cloned()
        .filter(|member| member_is_authoritative_follow_source(member, &world.dcs.value.cache, world.dcs.updated_at));
    let local_member = world.dcs.value.cache.members.get(&self_member_id).cloned();
    let promotion_safety = local_member
        .as_ref()
        .map(|member| {
            evaluate_promotion_safety(
                &member_promotion_candidate(member),
                &world.dcs.value.cache,
                world.dcs.updated_at,
            )
        })
        .unwrap_or_else(PromotionSafety::safe);

    ReconcileFacts {
        self_member_id: self_member_id.clone(),
        cluster_mode,
        leadership_transfer: current.leadership_transfer.clone(),
        current_process: world.process.value.clone(),
        local_member,
        trust: world.dcs.value.trust.clone(),
        switchover_pending: world.dcs.value.cache.switchover.is_some(),
        switchover_target: world
            .dcs
            .value
            .cache
            .switchover
            .as_ref()
            .and_then(|request| request.switchover_to.clone()),
        expected_cluster_identity: world
            .dcs
            .value
            .cache
            .cluster_identity
            .as_ref()
            .map(|record| record.system_identifier),
        authoritative_leader_member_id: authoritative_leader_member_id.clone(),
        authoritative_leader_member,
        i_am_authoritative_leader: authoritative_leader_member_id.as_ref() == Some(&self_member_id),
        postgres_reachable: is_postgres_reachable(&world.pg.value),
        postgres_primary: is_local_primary(&world.pg.value),
        postgres_replica: is_local_replica(&world.pg.value),
        promotion_safety,
        elected_candidate: highest_ranked_candidate(
            &world.dcs.value.cache,
            world.dcs.updated_at,
            world
                .dcs
                .value
                .cache
                .cluster_identity
                .as_ref()
                .map(|record| record.system_identifier),
        ),
    }
}

pub(crate) fn derive_cluster_mode(
    dcs: &crate::dcs::state::DcsState,
    _self_member_id: &MemberId,
) -> ClusterMode {
    match (
        &dcs.trust,
        dcs.cache.cluster_initialized.as_ref(),
        dcs.cache.bootstrap_lock.as_ref(),
        dcs.cache.leader.as_ref(),
    ) {
        (DcsTrust::NotTrusted, _, _, _) => ClusterMode::DcsUnavailable,
        (_, None, Some(record), _) => ClusterMode::UninitializedBootstrapInProgress {
            holder: record.holder.clone(),
        },
        (_, None, None, _) => ClusterMode::UninitializedNoBootstrapOwner,
        (DcsTrust::FreshQuorum, Some(_), _, Some(record)) => ClusterMode::InitializedLeaderPresent {
            leader: record.member_id.clone(),
        },
        (DcsTrust::FreshQuorum, Some(_), _, None) => ClusterMode::InitializedNoLeaderFreshQuorum,
        (_, Some(_), _, _) => ClusterMode::InitializedNoLeaderNoFreshQuorum,
    }
}

pub(crate) fn desired_state_for(facts: &ReconcileFacts) -> DesiredNodeState {
    if cluster_identity_missing_in_initialized_cluster(facts) {
        return quiescent(QuiescentReason::WaitingForRecoveryPreconditions);
    }

    match &facts.cluster_mode {
        ClusterMode::DcsUnavailable => desired_state_when_dcs_unavailable(facts),
        ClusterMode::UninitializedNoBootstrapOwner => desired_state_when_uninitialized_without_owner(facts),
        ClusterMode::UninitializedBootstrapInProgress { holder } => {
            desired_state_when_bootstrap_in_progress(facts, holder)
        }
        ClusterMode::InitializedLeaderPresent { leader } => {
            desired_state_with_authoritative_leader(facts, leader)
        }
        ClusterMode::InitializedNoLeaderFreshQuorum => {
            desired_state_without_leader_with_fresh_quorum(facts)
        }
        ClusterMode::InitializedNoLeaderNoFreshQuorum => {
            desired_state_without_leader_without_fresh_quorum(facts)
        }
    }
}

pub(crate) fn next_leadership_transfer(
    current: &LeadershipTransferState,
    facts: &ReconcileFacts,
) -> LeadershipTransferState {
    match (
        facts.switchover_pending,
        facts.i_am_authoritative_leader,
        facts.authoritative_leader_member_id.as_ref(),
        current,
    ) {
        (false, _, _, _) => LeadershipTransferState::None,
        (true, true, _, _) => LeadershipTransferState::WaitingForOtherLeader {
            target: facts.switchover_target.clone(),
        },
        (
            true,
            false,
            Some(other_leader),
            LeadershipTransferState::WaitingForOtherLeader { .. },
        ) if other_leader != &facts.self_member_id => LeadershipTransferState::None,
        _ => current.clone(),
    }
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
                world
                    .dcs
                    .value
                    .cache
                    .cluster_identity
                    .as_ref()
                    .map(|record| record.system_identifier),
            )
        })
        .map(|member| member.member_id.clone())
        .collect()
}

pub(crate) fn switchover_target_is_eligible_member(
    member: &MemberRecord,
    cache: &DcsView,
    observed_at: UnixMillis,
    expected_cluster_identity: Option<SystemIdentifier>,
) -> bool {
    member_record_is_fresh(member, cache, observed_at)
        && member_matches_expected_identity(member, expected_cluster_identity)
        && match member.postgres_runtime_class {
            Some(PostgresRuntimeClass::RunningHealthy) => {
                evaluate_promotion_safety(&member_promotion_candidate(member), cache, observed_at)
                    .allows_promotion()
            }
            Some(PostgresRuntimeClass::OfflineInspectable) => {
                member.state_class == Some(MemberStateClass::Promotable)
            }
            _ => false,
        }
}

pub(crate) fn process_activity(
    process: &ProcessState,
    expected_kinds: &[ActiveJobKind],
) -> ProcessActivity {
    match process {
        ProcessState::Running { active, .. } => {
            if expected_kinds.contains(&active.kind) {
                ProcessActivity::Running
            } else {
                ProcessActivity::IdleNoOutcome
            }
        }
        ProcessState::Idle {
            last_outcome: Some(JobOutcome::Success { job_kind, .. }),
            ..
        } => {
            if expected_kinds.contains(job_kind) {
                ProcessActivity::IdleSuccess
            } else {
                ProcessActivity::IdleNoOutcome
            }
        }
        ProcessState::Idle {
            last_outcome:
                Some(JobOutcome::Failure { job_kind, .. } | JobOutcome::Timeout { job_kind, .. }),
            ..
        } => {
            if expected_kinds.contains(job_kind) {
                ProcessActivity::IdleFailure
            } else {
                ProcessActivity::IdleNoOutcome
            }
        }
        ProcessState::Idle {
            last_outcome: None, ..
        } => ProcessActivity::IdleNoOutcome,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessActivity {
    Running,
    IdleNoOutcome,
    IdleSuccess,
    IdleFailure,
}

fn desired_state_when_dcs_unavailable(facts: &ReconcileFacts) -> DesiredNodeState {
    if facts.postgres_primary {
        return fence();
    }
    quiescent(QuiescentReason::WaitingForAuthoritativeClusterState)
}

fn desired_state_when_uninitialized_without_owner(facts: &ReconcileFacts) -> DesiredNodeState {
    match local_uninitialized_state(facts) {
        LocalUninitializedState::EligibleForBootstrap => DesiredNodeState::Bootstrap {
            plan: BootstrapPlan::InitDb,
        },
        LocalUninitializedState::UnsafeUnexpectedDataDir => {
            quiescent(QuiescentReason::UnsafeUninitializedPgData)
        }
        LocalUninitializedState::Unknown => {
            quiescent(QuiescentReason::WaitingForAuthoritativeClusterState)
        }
    }
}

fn desired_state_when_bootstrap_in_progress(
    facts: &ReconcileFacts,
    holder: &MemberId,
) -> DesiredNodeState {
    if holder == &facts.self_member_id {
        return desired_state_when_uninitialized_without_owner(facts);
    }
    quiescent(QuiescentReason::WaitingForBootstrapWinner)
}

fn desired_state_with_authoritative_leader(
    facts: &ReconcileFacts,
    leader: &MemberId,
) -> DesiredNodeState {
    if facts.i_am_authoritative_leader {
        if facts.switchover_pending {
            return quiescent(QuiescentReason::WaitingForAuthoritativeLeader);
        }
        return primary_state_for_candidate(facts);
    }

    if matches!(
        facts.leadership_transfer,
        LeadershipTransferState::WaitingForOtherLeader { .. }
    ) && facts.authoritative_leader_member_id.as_ref() != Some(&facts.self_member_id)
    {
        return replica_or_quiescent_for_leader(facts, leader);
    }

    replica_or_quiescent_for_leader(facts, leader)
}

fn desired_state_without_leader_with_fresh_quorum(facts: &ReconcileFacts) -> DesiredNodeState {
    if matches!(
        facts.leadership_transfer,
        LeadershipTransferState::WaitingForOtherLeader { .. }
    ) {
        return quiescent(QuiescentReason::WaitingForAuthoritativeLeader);
    }

    if facts.switchover_pending {
        return desired_state_during_switchover_without_leader(facts);
    }

    match facts.elected_candidate.as_ref() {
        Some(candidate) if candidate == &facts.self_member_id => primary_state_for_candidate(facts),
        _ if facts.postgres_primary => fence(),
        _ => quiescent(QuiescentReason::WaitingForAuthoritativeLeader),
    }
}

fn desired_state_during_switchover_without_leader(facts: &ReconcileFacts) -> DesiredNodeState {
    match facts.switchover_target.as_ref() {
        Some(target) if target == &facts.self_member_id => {
            if local_target_is_eligible(facts) {
                primary_state_for_candidate(facts)
            } else {
                quiescent(QuiescentReason::WaitingForRecoveryPreconditions)
            }
        }
        Some(_) => quiescent(QuiescentReason::WaitingForAuthoritativeLeader),
        None if facts.postgres_primary => quiescent(QuiescentReason::WaitingForAuthoritativeLeader),
        None => match facts.elected_candidate.as_ref() {
            Some(candidate) if candidate == &facts.self_member_id => primary_state_for_candidate(facts),
            _ => quiescent(QuiescentReason::WaitingForAuthoritativeLeader),
        },
    }
}

fn desired_state_without_leader_without_fresh_quorum(facts: &ReconcileFacts) -> DesiredNodeState {
    if facts.postgres_primary {
        return fence();
    }
    quiescent(QuiescentReason::WaitingForFreshQuorum)
}

fn primary_state_for_candidate(facts: &ReconcileFacts) -> DesiredNodeState {
    if !local_matches_expected_identity(facts) || local_state_is_invalid(facts) {
        return fence();
    }

    match (facts.i_am_authoritative_leader, facts.postgres_primary, facts.postgres_replica) {
        (true, true, _) => DesiredNodeState::Primary {
            plan: PrimaryPlan::KeepLeader,
        },
        (_, true, _) => DesiredNodeState::Primary {
            plan: PrimaryPlan::AcquireLeaderThenResumePrimary,
        },
        (_, _, true) if facts.promotion_safety.allows_promotion() => DesiredNodeState::Primary {
            plan: PrimaryPlan::AcquireLeaderThenPromote,
        },
        _ if local_can_start_primary(facts) => DesiredNodeState::Primary {
            plan: PrimaryPlan::AcquireLeaderThenStartPrimary,
        },
        _ => quiescent(QuiescentReason::WaitingForRecoveryPreconditions),
    }
}

fn replica_or_quiescent_for_leader(
    facts: &ReconcileFacts,
    leader: &MemberId,
) -> DesiredNodeState {
    replica_plan_for_leader(facts, leader)
        .map(|plan| DesiredNodeState::Replica { plan })
        .unwrap_or_else(|| quiescent(QuiescentReason::WaitingForRecoveryPreconditions))
}

fn replica_plan_for_leader(
    facts: &ReconcileFacts,
    leader: &MemberId,
) -> Option<ReplicaPlan> {
    let leader_member = facts.authoritative_leader_member.as_ref()?;
    if &leader_member.member_id != leader {
        return None;
    }
    if local_state_is_invalid(facts) {
        return None;
    }
    if !local_matches_expected_identity(facts) {
        return Some(ReplicaPlan::BasebackupThenFollow {
            leader_member_id: leader.clone(),
        });
    }
    if local_data_dir_missing_or_empty(facts) {
        return Some(ReplicaPlan::BasebackupThenFollow {
            leader_member_id: leader.clone(),
        });
    }
    if rewind_failed_recently(&facts.current_process) {
        return Some(ReplicaPlan::BasebackupThenFollow {
            leader_member_id: leader.clone(),
        });
    }
    if facts.postgres_primary || local_member_claims_primary(facts) {
        return Some(ReplicaPlan::RewindThenFollow {
            leader_member_id: leader.clone(),
        });
    }
    Some(ReplicaPlan::DirectFollow {
        leader_member_id: leader.clone(),
    })
}

fn highest_ranked_candidate(
    cache: &DcsView,
    observed_at: UnixMillis,
    expected_cluster_identity: Option<SystemIdentifier>,
) -> Option<MemberId> {
    cache
        .members
        .values()
        .filter(|member| member_record_is_fresh(member, cache, observed_at))
        .map(|member| CandidateDescriptor {
            member: member.clone(),
            identity_rank: identity_rank(member, expected_cluster_identity),
            eligible: switchover_target_is_eligible_member(
                member,
                cache,
                observed_at,
                expected_cluster_identity,
            ),
            timeline_rank: member.timeline.map(|value| u64::from(value.0)).unwrap_or(0),
            lsn_rank: member
                .durable_end_lsn
                .or(member.write_lsn)
                .or(member.replay_lsn)
                .map(|value| value.0)
                .unwrap_or(0),
            runtime_rank: runtime_rank(member),
        })
        .max_by(compare_candidate_descriptors)
        .and_then(|descriptor| {
            if descriptor.identity_rank == 2 && descriptor.eligible {
                Some(descriptor.member.member_id)
            } else {
                None
            }
        })
}

fn compare_candidate_descriptors(left: &CandidateDescriptor, right: &CandidateDescriptor) -> Ordering {
    left.identity_rank
        .cmp(&right.identity_rank)
        .then_with(|| left.eligible.cmp(&right.eligible))
        .then_with(|| left.timeline_rank.cmp(&right.timeline_rank))
        .then_with(|| left.lsn_rank.cmp(&right.lsn_rank))
        .then_with(|| left.runtime_rank.cmp(&right.runtime_rank))
        .then_with(|| right.member.member_id.cmp(&left.member.member_id))
}

fn identity_rank(member: &MemberRecord, expected_cluster_identity: Option<SystemIdentifier>) -> u8 {
    match (member.system_identifier, expected_cluster_identity) {
        (Some(actual), Some(expected)) if actual == expected => 2,
        (None, Some(_)) => 1,
        (Some(_), Some(_)) => 0,
        (_, None) => 0,
    }
}

fn runtime_rank(member: &MemberRecord) -> u8 {
    match member.postgres_runtime_class {
        Some(PostgresRuntimeClass::RunningHealthy) => 2,
        Some(PostgresRuntimeClass::OfflineInspectable) => 1,
        _ => 0,
    }
}

fn cluster_identity_missing_in_initialized_cluster(facts: &ReconcileFacts) -> bool {
    matches!(
        facts.cluster_mode,
        ClusterMode::InitializedLeaderPresent { .. }
            | ClusterMode::InitializedNoLeaderFreshQuorum
            | ClusterMode::InitializedNoLeaderNoFreshQuorum
    ) && facts.expected_cluster_identity.is_none()
}

fn member_matches_expected_identity(
    member: &MemberRecord,
    expected_cluster_identity: Option<SystemIdentifier>,
) -> bool {
    match (member.system_identifier, expected_cluster_identity) {
        (Some(actual), Some(expected)) => actual == expected,
        _ => false,
    }
}

fn member_is_authoritative_follow_source(
    member: &MemberRecord,
    cache: &DcsView,
    observed_at: UnixMillis,
) -> bool {
    member_record_is_fresh(member, cache, observed_at)
        && member.role == MemberRole::Primary
        && member.sql == SqlStatus::Healthy
        && member.readiness == Readiness::Ready
}

fn local_target_is_eligible(facts: &ReconcileFacts) -> bool {
    facts
        .local_member
        .as_ref()
        .map(|member| {
            member_matches_expected_identity(member, facts.expected_cluster_identity)
                && match member.postgres_runtime_class {
                    Some(PostgresRuntimeClass::RunningHealthy) => {
                        facts.promotion_safety.allows_promotion()
                    }
                    Some(PostgresRuntimeClass::OfflineInspectable) => {
                        member.state_class == Some(MemberStateClass::Promotable)
                    }
                    _ => false,
                }
        })
        .unwrap_or(false)
}

enum LocalUninitializedState {
    EligibleForBootstrap,
    UnsafeUnexpectedDataDir,
    Unknown,
}

fn local_uninitialized_state(facts: &ReconcileFacts) -> LocalUninitializedState {
    match facts.local_member.as_ref().and_then(|member| member.state_class.clone()) {
        Some(MemberStateClass::EmptyOrMissingDataDir) => LocalUninitializedState::EligibleForBootstrap,
        Some(_) => LocalUninitializedState::UnsafeUnexpectedDataDir,
        None => LocalUninitializedState::Unknown,
    }
}

fn local_matches_expected_identity(facts: &ReconcileFacts) -> bool {
    facts
        .local_member
        .as_ref()
        .map(|member| member_matches_expected_identity(member, facts.expected_cluster_identity))
        .unwrap_or(false)
}

fn local_data_dir_missing_or_empty(facts: &ReconcileFacts) -> bool {
    facts
        .local_member
        .as_ref()
        .and_then(|member| member.state_class.clone())
        == Some(MemberStateClass::EmptyOrMissingDataDir)
}

fn local_state_is_invalid(facts: &ReconcileFacts) -> bool {
    facts
        .local_member
        .as_ref()
        .and_then(|member| member.state_class.clone())
        == Some(MemberStateClass::InvalidDataDir)
}

fn local_can_start_primary(facts: &ReconcileFacts) -> bool {
    facts
        .local_member
        .as_ref()
        .map(|member| {
            member.postgres_runtime_class == Some(PostgresRuntimeClass::OfflineInspectable)
                && member.state_class == Some(MemberStateClass::Promotable)
        })
        .unwrap_or(false)
}

fn local_member_claims_primary(facts: &ReconcileFacts) -> bool {
    facts
        .local_member
        .as_ref()
        .map(|member| member.role == MemberRole::Primary)
        .unwrap_or(false)
}

fn rewind_failed_recently(process: &ProcessState) -> bool {
    matches!(
        process,
        ProcessState::Idle {
            last_outcome:
                Some(JobOutcome::Failure {
                    job_kind: ActiveJobKind::PgRewind,
                    ..
                } | JobOutcome::Timeout {
                    job_kind: ActiveJobKind::PgRewind,
                    ..
                }),
            ..
        }
    )
}

fn quiescent(reason: QuiescentReason) -> DesiredNodeState {
    DesiredNodeState::Quiescent { reason }
}

fn fence() -> DesiredNodeState {
    DesiredNodeState::Fence {
        plan: FencePlan::StopAndStayNonWritable,
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

fn is_local_replica(state: &PgInfoState) -> bool {
    matches!(
        state,
        PgInfoState::Replica {
            common,
            ..
        } if matches!(common.sql, SqlStatus::Healthy)
    )
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
    cache: &crate::dcs::state::DcsView,
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
    cache: &crate::dcs::state::DcsView,
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
        .filter_map(|member| member.timeline.map(|timeline| (timeline, member.member_id.clone())))
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::RuntimeConfig,
        dcs::state::{
            ClusterIdentityRecord, ClusterInitializedRecord, DcsState, DcsTrust, DcsView,
            LeaderRecord, MemberRecord, MemberRole, MemberStateClass, PostgresRuntimeClass,
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{MemberId, SystemIdentifier, TimelineId, UnixMillis, Version, Versioned, WorkerStatus, WalLsn},
    };

    use super::{desired_state_for, eligible_switchover_targets, next_leadership_transfer, reconcile_facts};
    use crate::ha::state::{
        ClusterMode, DesiredNodeState, FencePlan, HaState, LeadershipTransferState, QuiescentReason,
        WorldSnapshot,
    };

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_member(member_id: &str) -> MemberRecord {
        MemberRecord {
            member_id: MemberId(member_id.to_string()),
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            api_url: None,
            role: MemberRole::Replica,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: Some(TimelineId(1)),
            write_lsn: None,
            replay_lsn: Some(WalLsn(10)),
            system_identifier: Some(SystemIdentifier(42)),
            durable_end_lsn: Some(WalLsn(10)),
            state_class: Some(MemberStateClass::ReplicaOnly),
            postgres_runtime_class: Some(PostgresRuntimeClass::RunningHealthy),
            updated_at: UnixMillis(100),
            pg_version: Version(16),
        }
    }

    fn sample_world(cache: DcsView, trust: DcsTrust, pg: PgInfoState) -> WorldSnapshot {
        let now = UnixMillis(100);
        let config = cache.config.clone();
        WorldSnapshot {
            config: Versioned::new(Version(1), now, config),
            pg: Versioned::new(Version(1), now, pg),
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

    fn sample_pg_replica() -> PgInfoState {
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
                last_refresh_at: Some(UnixMillis(100)),
            },
            replay_lsn: WalLsn(10),
            follow_lsn: None,
            upstream: None,
        }
    }

    fn sample_pg_primary() -> PgInfoState {
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
                last_refresh_at: Some(UnixMillis(100)),
            },
            wal_lsn: WalLsn(11),
            slots: Vec::new(),
        }
    }

    #[test]
    fn authoritative_leader_absence_fences_local_primary_without_fresh_quorum() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = DcsView {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            cluster_initialized: Some(ClusterInitializedRecord {
                initialized_by: self_id.clone(),
                initialized_at: UnixMillis(10),
            }),
            cluster_identity: Some(ClusterIdentityRecord {
                system_identifier: SystemIdentifier(42),
                bootstrapped_by: self_id.clone(),
                bootstrapped_at: UnixMillis(10),
            }),
            bootstrap_lock: None,
        };
        let mut local = sample_member("node-a");
        local.role = MemberRole::Primary;
        local.write_lsn = Some(WalLsn(11));
        local.replay_lsn = None;
        cache.members.insert(self_id.clone(), local);
        let world = sample_world(cache, DcsTrust::NoFreshQuorum, sample_pg_primary());
        let current = HaState {
            worker: WorkerStatus::Running,
            cluster_mode: ClusterMode::DcsUnavailable,
            desired_state: DesiredNodeState::Quiescent {
                reason: QuiescentReason::WaitingForFreshQuorum,
            },
            leadership_transfer: LeadershipTransferState::None,
            tick: 0,
        };

        let facts = reconcile_facts(&current, &world);
        assert_eq!(
            desired_state_for(&facts),
            DesiredNodeState::Fence {
                plan: FencePlan::StopAndStayNonWritable,
            }
        );
    }

    #[test]
    fn targeted_switchover_blocks_non_target_candidates() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = DcsView {
            members: BTreeMap::new(),
            leader: None,
            switchover: Some(crate::dcs::state::SwitchoverRequest {
                switchover_to: Some(MemberId("node-b".to_string())),
            }),
            config: sample_runtime_config(),
            cluster_initialized: Some(ClusterInitializedRecord {
                initialized_by: self_id.clone(),
                initialized_at: UnixMillis(10),
            }),
            cluster_identity: Some(ClusterIdentityRecord {
                system_identifier: SystemIdentifier(42),
                bootstrapped_by: self_id.clone(),
                bootstrapped_at: UnixMillis(10),
            }),
            bootstrap_lock: None,
        };
        cache.members.insert(self_id.clone(), sample_member("node-a"));
        cache.members.insert(MemberId("node-b".to_string()), sample_member("node-b"));
        let world = sample_world(cache, DcsTrust::FreshQuorum, sample_pg_replica());
        let current = HaState {
            worker: WorkerStatus::Running,
            cluster_mode: ClusterMode::InitializedNoLeaderFreshQuorum,
            desired_state: DesiredNodeState::Quiescent {
                reason: QuiescentReason::WaitingForAuthoritativeLeader,
            },
            leadership_transfer: LeadershipTransferState::None,
            tick: 0,
        };

        let facts = reconcile_facts(&current, &world);
        assert_eq!(
            desired_state_for(&facts),
            DesiredNodeState::Quiescent {
                reason: QuiescentReason::WaitingForAuthoritativeLeader,
            }
        );
    }

    #[test]
    fn switchover_eligible_targets_require_matching_cluster_identity() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = DcsView {
            members: BTreeMap::new(),
            leader: Some(LeaderRecord {
                member_id: self_id.clone(),
            }),
            switchover: None,
            config: sample_runtime_config(),
            cluster_initialized: Some(ClusterInitializedRecord {
                initialized_by: self_id.clone(),
                initialized_at: UnixMillis(10),
            }),
            cluster_identity: Some(ClusterIdentityRecord {
                system_identifier: SystemIdentifier(42),
                bootstrapped_by: self_id.clone(),
                bootstrapped_at: UnixMillis(10),
            }),
            bootstrap_lock: None,
        };
        let leader = sample_member("node-a");
        let mut healthy_replica = sample_member("node-b");
        let mut wrong_cluster = sample_member("node-c");
        wrong_cluster.system_identifier = Some(SystemIdentifier(77));
        cache.members.insert(self_id, leader);
        cache.members
            .insert(MemberId("node-b".to_string()), healthy_replica.clone());
        cache.members
            .insert(MemberId("node-c".to_string()), wrong_cluster);
        let world = sample_world(cache, DcsTrust::FreshQuorum, sample_pg_primary());

        let eligible = eligible_switchover_targets(&world);
        assert!(eligible.contains(&MemberId("node-b".to_string())));
        assert!(!eligible.contains(&MemberId("node-c".to_string())));

        healthy_replica.replay_lsn = Some(WalLsn(1));
    }

    #[test]
    fn leadership_transfer_clears_once_other_leader_appears() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = DcsView {
            members: BTreeMap::new(),
            leader: Some(LeaderRecord {
                member_id: MemberId("node-b".to_string()),
            }),
            switchover: Some(crate::dcs::state::SwitchoverRequest { switchover_to: None }),
            config: sample_runtime_config(),
            cluster_initialized: Some(ClusterInitializedRecord {
                initialized_by: self_id.clone(),
                initialized_at: UnixMillis(10),
            }),
            cluster_identity: Some(ClusterIdentityRecord {
                system_identifier: SystemIdentifier(42),
                bootstrapped_by: self_id.clone(),
                bootstrapped_at: UnixMillis(10),
            }),
            bootstrap_lock: None,
        };
        cache.members.insert(MemberId("node-a".to_string()), sample_member("node-a"));
        let mut new_leader = sample_member("node-b");
        new_leader.role = MemberRole::Primary;
        new_leader.write_lsn = Some(WalLsn(12));
        new_leader.replay_lsn = None;
        cache.members.insert(MemberId("node-b".to_string()), new_leader);
        let world = sample_world(cache, DcsTrust::FreshQuorum, sample_pg_replica());
        let current = HaState {
            worker: WorkerStatus::Running,
            cluster_mode: ClusterMode::InitializedNoLeaderFreshQuorum,
            desired_state: DesiredNodeState::Quiescent {
                reason: QuiescentReason::WaitingForAuthoritativeLeader,
            },
            leadership_transfer: LeadershipTransferState::WaitingForOtherLeader { target: None },
            tick: 0,
        };

        let facts = reconcile_facts(&current, &world);
        assert_eq!(
            next_leadership_transfer(&current.leadership_transfer, &facts),
            LeadershipTransferState::None
        );
    }
}
