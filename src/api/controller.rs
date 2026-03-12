use serde::{Deserialize, Serialize};

use crate::{
    api::{
        AcceptedResponse, ApiError, ApiResult, AuthorityProjectionResponse, CandidacyResponse,
        DcsTrustResponse, FailSafeGoalResponse, FenceCutoffResponse, FenceReasonResponse,
        FollowGoalResponse, HaClusterMemberResponse, HaCommandResponse, HaStateResponse,
        IdleReasonResponse, IneligibleReasonResponse, LeaseEpochResponse, MemberRoleResponse,
        NoPrimaryReasonResponse, ReadinessResponse, RecoveryPlanResponse, RoleIntentResponse,
        ShutdownModeResponse, SqlStatusResponse, SwitchoverBlockerResponse,
        SwitchoverIntentResponse,
    },
    config::RuntimeConfig,
    dcs::{
        state::{
            DcsState, DcsTrust, MemberPostgresView, MemberSlot, SwitchoverIntentRecord,
            SwitchoverTargetRecord,
        },
        store::DcsStore,
    },
    ha::{
        state::HaState,
        types::{
            AuthorityView, Candidacy, FailSafeGoal, FenceCutoff, FenceReason, FollowGoal,
            IdleReason, IneligibleReason, NoPrimaryReason, PublicationGoal, ReconcileAction,
            ShutdownMode, SwitchoverBlocker, TargetRole,
        },
    },
    pginfo::state::Readiness,
    state::{MemberId, Versioned},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {
    #[serde(default)]
    pub(crate) switchover_to: Option<String>,
}

pub(crate) fn post_switchover(
    scope: &str,
    self_id: &MemberId,
    store: &mut dyn DcsStore,
    dcs: &DcsState,
    ha: &HaState,
    input: SwitchoverRequestInput,
) -> ApiResult<AcceptedResponse> {
    let request = validate_switchover_request(self_id, dcs, ha, input)?;
    let encoded = serde_json::to_string(&request)
        .map_err(|err| ApiError::internal(format!("switchover encode failed: {err}")))?;
    let path = format!("/{}/switchover", scope.trim_matches('/'));
    store
        .write_path(&path, encoded)
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;

    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn delete_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
) -> ApiResult<AcceptedResponse> {
    store
        .delete_path(format!("/{}/switchover", scope.trim_matches('/')).as_str())
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;
    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn get_ha_state(
    cfg: &RuntimeConfig,
    dcs: &Versioned<DcsState>,
    ha: &Versioned<HaState>,
) -> HaStateResponse {
    HaStateResponse {
        cluster_name: cfg.cluster.name.clone(),
        scope: cfg.dcs.scope.clone(),
        self_member_id: cfg.cluster.member_id.clone(),
        leader_lease_holder: dcs
            .value
            .cache
            .leader_lease
            .as_ref()
            .map(|leader| leader.holder.0.clone()),
        switchover: dcs
            .value
            .cache
            .switchover_intent
            .as_ref()
            .map(map_switchover_intent),
        member_slot_count: dcs.value.cache.member_slots.len(),
        member_slots: dcs
            .value
            .cache
            .member_slots
            .values()
            .map(map_member_slot)
            .collect(),
        dcs_trust: map_dcs_trust(&dcs.value.trust),
        authority_projection: map_authority(&ha.value.publication.authority),
        fence_cutoff: ha
            .value
            .publication
            .fence_cutoff
            .as_ref()
            .map(map_fence_cutoff),
        role_intent: map_role(&ha.value.role),
        ha_tick: ha.value.tick,
        planned_commands: ha
            .value
            .planned_commands
            .iter()
            .map(map_command)
            .collect(),
        snapshot_sequence: dcs.version.0.max(ha.version.0),
    }
}

fn validate_switchover_request(
    self_id: &MemberId,
    dcs: &DcsState,
    ha: &HaState,
    input: SwitchoverRequestInput,
) -> ApiResult<SwitchoverIntentRecord> {
    if dcs.trust != DcsTrust::FullQuorum {
        return Err(ApiError::bad_request(
            "switchover requests require full quorum DCS trust".to_string(),
        ));
    }

    match &ha.publication.authority {
        AuthorityView::Primary { member, .. } if member == self_id => {}
        _ => {
            return Err(ApiError::bad_request(
                "switchover requests must be sent to the authoritative primary".to_string(),
            ));
        }
    }

    let Some(raw_target) = input.switchover_to else {
        let target_member_id = select_generic_switchover_target(self_id, dcs).ok_or_else(|| {
            ApiError::bad_request(
                "no eligible switchover target is currently available".to_string(),
            )
        })?;
        return Ok(SwitchoverIntentRecord {
            target: SwitchoverTargetRecord::Specific(target_member_id),
        });
    };

    let target = raw_target.trim();
    if target.is_empty() {
        return Err(ApiError::bad_request(
            "switchover_to must not be empty".to_string(),
        ));
    }

    let target_member_id = MemberId(target.to_string());
    if &target_member_id == self_id {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is already the leader"
        )));
    }

    let target_member = dcs
        .cache
        .member_slots
        .get(&target_member_id)
        .ok_or_else(|| ApiError::bad_request(format!("unknown switchover_to member `{target}`")))?;
    if !member_slot_is_eligible_target(target_member) {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is not an eligible switchover target"
        )));
    }

    Ok(SwitchoverIntentRecord {
        target: SwitchoverTargetRecord::Specific(target_member_id),
    })
}

fn member_slot_is_eligible_target(value: &MemberSlot) -> bool {
    match &value.postgres {
        MemberPostgresView::Unknown(observation) => observation.readiness == Readiness::Ready,
        MemberPostgresView::Primary(_) => false,
        MemberPostgresView::Replica(observation) => observation.readiness == Readiness::Ready,
    }
}

fn select_generic_switchover_target(self_id: &MemberId, dcs: &DcsState) -> Option<MemberId> {
    dcs.cache
        .member_slots
        .iter()
        .filter(|(member_id, member)| *member_id != self_id && member_slot_is_eligible_target(member))
        .max_by(|(left_id, left_member), (right_id, right_member)| {
            compare_switchover_target_slots(left_id, left_member, right_id, right_member)
        })
        .map(|(member_id, _)| member_id.clone())
}

fn compare_switchover_target_slots(
    left_id: &MemberId,
    left_member: &MemberSlot,
    right_id: &MemberId,
    right_member: &MemberSlot,
) -> std::cmp::Ordering {
    target_rank(left_member)
        .cmp(&target_rank(right_member))
        .then_with(|| right_id.cmp(left_id))
}

fn target_rank(member: &MemberSlot) -> (u8, u64, u64) {
    match &member.postgres {
        MemberPostgresView::Replica(observation) => observation
            .replay_wal
            .as_ref()
            .or(observation.follow_wal.as_ref())
            .map(|wal| (1, wal.timeline.map_or(0, |value| u64::from(value.0)), wal.lsn.0))
            .unwrap_or((0, 0, 0)),
        MemberPostgresView::Unknown(observation) => {
            (0, observation.timeline.map_or(0, |value| u64::from(value.0)), 0)
        }
        MemberPostgresView::Primary(_) => (0, 0, 0),
    }
}

fn map_dcs_trust(value: &DcsTrust) -> DcsTrustResponse {
    match value {
        DcsTrust::FullQuorum => DcsTrustResponse::FullQuorum,
        DcsTrust::Degraded => DcsTrustResponse::Degraded,
        DcsTrust::NotTrusted => DcsTrustResponse::NotTrusted,
    }
}

fn map_switchover_intent(value: &SwitchoverIntentRecord) -> SwitchoverIntentResponse {
    match &value.target {
        SwitchoverTargetRecord::AnyHealthyReplica => SwitchoverIntentResponse::AnyHealthyReplica,
        SwitchoverTargetRecord::Specific(member_id) => SwitchoverIntentResponse::Specific {
            member_id: member_id.0.clone(),
        },
    }
}

fn map_member_slot(value: &MemberSlot) -> HaClusterMemberResponse {
    HaClusterMemberResponse {
        member_id: value.lease.owner.0.clone(),
        postgres_host: value.routing.postgres.host.clone(),
        postgres_port: value.routing.postgres.port,
        api_url: value.routing.api.as_ref().map(|endpoint| endpoint.url.clone()),
        role: map_member_role(&value.postgres),
        sql: map_sql_status(&value.postgres),
        readiness: map_readiness(&value.postgres),
        timeline: member_timeline(value),
        write_lsn: member_write_lsn(value),
        replay_lsn: member_replay_lsn(value),
        pg_version: member_pg_version(value).0,
    }
}

fn member_timeline(value: &MemberSlot) -> Option<u64> {
    match &value.postgres {
        MemberPostgresView::Unknown(observation) => observation.timeline.map(|value| u64::from(value.0)),
        MemberPostgresView::Primary(observation) => {
            observation.committed_wal.timeline.map(|value| u64::from(value.0))
        }
        MemberPostgresView::Replica(observation) => observation
            .replay_wal
            .as_ref()
            .and_then(|wal| wal.timeline.map(|value| u64::from(value.0))),
    }
}

fn member_write_lsn(value: &MemberSlot) -> Option<u64> {
    match &value.postgres {
        MemberPostgresView::Primary(observation) => Some(observation.committed_wal.lsn.0),
        MemberPostgresView::Unknown(_) | MemberPostgresView::Replica(_) => None,
    }
}

fn member_replay_lsn(value: &MemberSlot) -> Option<u64> {
    match &value.postgres {
        MemberPostgresView::Replica(observation) => {
            observation.replay_wal.as_ref().map(|wal| wal.lsn.0)
        }
        MemberPostgresView::Unknown(_) | MemberPostgresView::Primary(_) => None,
    }
}

fn member_pg_version(value: &MemberSlot) -> crate::state::Version {
    match &value.postgres {
        MemberPostgresView::Unknown(observation) => observation.pg_version,
        MemberPostgresView::Primary(observation) => observation.pg_version,
        MemberPostgresView::Replica(observation) => observation.pg_version,
    }
}

fn map_member_role(value: &MemberPostgresView) -> MemberRoleResponse {
    match value {
        MemberPostgresView::Unknown(_) => MemberRoleResponse::Unknown,
        MemberPostgresView::Primary(_) => MemberRoleResponse::Primary,
        MemberPostgresView::Replica(_) => MemberRoleResponse::Replica,
    }
}

fn map_sql_status(value: &MemberPostgresView) -> SqlStatusResponse {
    match value {
        MemberPostgresView::Unknown(_) => SqlStatusResponse::Unknown,
        MemberPostgresView::Primary(_) | MemberPostgresView::Replica(_) => SqlStatusResponse::Healthy,
    }
}

fn map_readiness(value: &MemberPostgresView) -> ReadinessResponse {
    match value {
        MemberPostgresView::Unknown(observation) => map_readiness_value(&observation.readiness),
        MemberPostgresView::Primary(observation) => map_readiness_value(&observation.readiness),
        MemberPostgresView::Replica(observation) => map_readiness_value(&observation.readiness),
    }
}

fn map_readiness_value(value: &Readiness) -> ReadinessResponse {
    match value {
        Readiness::Unknown => ReadinessResponse::Unknown,
        Readiness::Ready => ReadinessResponse::Ready,
        Readiness::NotReady => ReadinessResponse::NotReady,
    }
}

fn map_authority(value: &AuthorityView) -> AuthorityProjectionResponse {
    match value {
        AuthorityView::Primary { member, epoch } => AuthorityProjectionResponse::Primary {
            member_id: member.0.clone(),
            epoch: map_epoch(epoch),
        },
        AuthorityView::NoPrimary(reason) => AuthorityProjectionResponse::NoPrimary {
            reason: map_no_primary_reason(reason),
        },
        AuthorityView::Unknown => AuthorityProjectionResponse::Unknown,
    }
}

fn map_epoch(value: &crate::ha::types::LeaseEpoch) -> LeaseEpochResponse {
    LeaseEpochResponse {
        holder: value.holder.0.clone(),
        generation: value.generation,
    }
}

fn map_fence_cutoff(value: &FenceCutoff) -> FenceCutoffResponse {
    FenceCutoffResponse {
        epoch: map_epoch(&value.epoch),
        committed_lsn: value.committed_lsn,
    }
}

fn map_no_primary_reason(value: &NoPrimaryReason) -> NoPrimaryReasonResponse {
    match value {
        NoPrimaryReason::DcsDegraded => NoPrimaryReasonResponse::DcsDegraded,
        NoPrimaryReason::LeaseOpen => NoPrimaryReasonResponse::LeaseOpen,
        NoPrimaryReason::Recovering => NoPrimaryReasonResponse::Recovering,
        NoPrimaryReason::SwitchoverRejected(blocker) => {
            NoPrimaryReasonResponse::SwitchoverRejected {
                blocker: map_switchover_blocker(blocker),
            }
        }
    }
}

fn map_switchover_blocker(value: &SwitchoverBlocker) -> SwitchoverBlockerResponse {
    match value {
        SwitchoverBlocker::TargetMissing => SwitchoverBlockerResponse::TargetMissing,
        SwitchoverBlocker::TargetIneligible(reason) => {
            SwitchoverBlockerResponse::TargetIneligible {
                reason: map_ineligible_reason(reason),
            }
        }
    }
}

fn map_role(value: &TargetRole) -> RoleIntentResponse {
    match value {
        TargetRole::Leader(epoch) => RoleIntentResponse::Leader {
            epoch: map_epoch(epoch),
        },
        TargetRole::Candidate(candidacy) => RoleIntentResponse::Candidate {
            candidacy: map_candidacy(candidacy),
        },
        TargetRole::Follower(goal) => RoleIntentResponse::Follower {
            goal: map_follow_goal(goal),
        },
        TargetRole::FailSafe(goal) => RoleIntentResponse::FailSafe {
            goal: map_fail_safe_goal(goal),
        },
        TargetRole::DemotingForSwitchover(member_id) => {
            RoleIntentResponse::DemotingForSwitchover {
                member_id: member_id.0.clone(),
            }
        }
        TargetRole::Fenced(reason) => RoleIntentResponse::Fenced {
            reason: map_fence_reason(reason),
        },
        TargetRole::Idle(reason) => RoleIntentResponse::Idle {
            reason: map_idle_reason(reason),
        },
    }
}

fn map_candidacy(value: &Candidacy) -> CandidacyResponse {
    match value {
        Candidacy::Bootstrap => CandidacyResponse::Bootstrap,
        Candidacy::Failover => CandidacyResponse::Failover,
        Candidacy::ResumeAfterOutage => CandidacyResponse::ResumeAfterOutage,
        Candidacy::TargetedSwitchover(member_id) => CandidacyResponse::TargetedSwitchover {
            member_id: member_id.0.clone(),
        },
    }
}

fn map_follow_goal(value: &FollowGoal) -> FollowGoalResponse {
    FollowGoalResponse {
        leader: value.leader.0.clone(),
        recovery: map_recovery_plan(&value.recovery),
    }
}

fn map_recovery_plan(value: &crate::ha::types::RecoveryPlan) -> RecoveryPlanResponse {
    match value {
        crate::ha::types::RecoveryPlan::None => RecoveryPlanResponse::None,
        crate::ha::types::RecoveryPlan::StartStreaming => RecoveryPlanResponse::StartStreaming,
        crate::ha::types::RecoveryPlan::Rewind => RecoveryPlanResponse::Rewind,
        crate::ha::types::RecoveryPlan::Basebackup => RecoveryPlanResponse::Basebackup,
    }
}

fn map_fail_safe_goal(value: &FailSafeGoal) -> FailSafeGoalResponse {
    match value {
        FailSafeGoal::PrimaryMustStop(cutoff) => FailSafeGoalResponse::PrimaryMustStop {
            cutoff: map_fence_cutoff(cutoff),
        },
        FailSafeGoal::ReplicaKeepFollowing(upstream) => {
            FailSafeGoalResponse::ReplicaKeepFollowing {
                upstream: upstream.as_ref().map(|value| value.0.clone()),
            }
        }
        FailSafeGoal::WaitForQuorum => FailSafeGoalResponse::WaitForQuorum,
    }
}

fn map_idle_reason(value: &IdleReason) -> IdleReasonResponse {
    match value {
        IdleReason::AwaitingLeader => IdleReasonResponse::AwaitingLeader,
        IdleReason::AwaitingTarget(member_id) => IdleReasonResponse::AwaitingTarget {
            member_id: member_id.0.clone(),
        },
    }
}

fn map_fence_reason(value: &FenceReason) -> FenceReasonResponse {
    match value {
        FenceReason::ForeignLeaderDetected => FenceReasonResponse::ForeignLeaderDetected,
        FenceReason::StorageStalled => FenceReasonResponse::StorageStalled,
    }
}

fn map_command(value: &ReconcileAction) -> HaCommandResponse {
    match value {
        ReconcileAction::InitDb => HaCommandResponse::InitDb,
        ReconcileAction::BaseBackup(member_id) => HaCommandResponse::BaseBackup {
            member_id: member_id.0.clone(),
        },
        ReconcileAction::PgRewind(member_id) => HaCommandResponse::PgRewind {
            member_id: member_id.0.clone(),
        },
        ReconcileAction::StartPrimary => HaCommandResponse::StartPrimary,
        ReconcileAction::StartReplica(member_id) => HaCommandResponse::StartReplica {
            member_id: member_id.0.clone(),
        },
        ReconcileAction::Promote => HaCommandResponse::Promote,
        ReconcileAction::Demote(mode) => HaCommandResponse::Demote {
            mode: map_shutdown_mode(mode),
        },
        ReconcileAction::AcquireLease(candidacy) => HaCommandResponse::AcquireLease {
            candidacy: map_candidacy(candidacy),
        },
        ReconcileAction::ReleaseLease => HaCommandResponse::ReleaseLease,
        ReconcileAction::EnsureRequiredRoles => HaCommandResponse::EnsureRequiredRoles,
        ReconcileAction::Publish(publication) => HaCommandResponse::Publish {
            projection: map_publication_goal(publication),
        },
        ReconcileAction::ClearSwitchover => HaCommandResponse::ClearSwitchover,
    }
}

fn map_publication_goal(value: &PublicationGoal) -> AuthorityProjectionResponse {
    match value {
        PublicationGoal::KeepCurrent => AuthorityProjectionResponse::Unknown,
        PublicationGoal::PublishPrimary { primary, epoch } => AuthorityProjectionResponse::Primary {
            member_id: primary.0.clone(),
            epoch: map_epoch(epoch),
        },
        PublicationGoal::PublishNoPrimary { reason, .. } => AuthorityProjectionResponse::NoPrimary {
            reason: map_no_primary_reason(reason),
        },
    }
}

fn map_shutdown_mode(value: &ShutdownMode) -> ShutdownModeResponse {
    match value {
        ShutdownMode::Fast => ShutdownModeResponse::Fast,
        ShutdownMode::Immediate => ShutdownModeResponse::Immediate,
    }
}

fn map_ineligible_reason(value: &IneligibleReason) -> IneligibleReasonResponse {
    match value {
        IneligibleReason::NotReady => IneligibleReasonResponse::NotReady,
        IneligibleReason::Lagging => IneligibleReasonResponse::Lagging,
        IneligibleReason::Partitioned => IneligibleReasonResponse::Partitioned,
        IneligibleReason::ApiUnavailable => IneligibleReasonResponse::ApiUnavailable,
        IneligibleReason::StartingUp => IneligibleReasonResponse::StartingUp,
    }
}
