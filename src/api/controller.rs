use serde::{Deserialize, Serialize};

use crate::{
    api::{
        AcceptedResponse, ApiError, ApiResult, CandidacyResponse, DcsTrustResponse,
        FailSafeGoalResponse, FenceCutoffResponse, FenceReasonResponse, FollowGoalResponse,
        HaAuthorityResponse, HaClusterMemberResponse, HaStateResponse, IdleReasonResponse,
        IneligibleReasonResponse, LeaseEpochResponse, MemberRoleResponse, NoPrimaryReasonResponse,
        ReadinessResponse, ReconcileActionResponse, RecoveryPlanResponse, ShutdownModeResponse,
        SqlStatusResponse, SwitchoverBlockerResponse, TargetRoleResponse,
    },
    dcs::{
        state::{DcsTrust, MemberRecord, MemberRole, SwitchoverRequest},
        store::DcsStore,
    },
    debug_api::snapshot::SystemSnapshot,
    ha::types::{
        Candidacy, FailSafeGoal, FenceCutoff, FenceReason, FollowGoal, IdleReason,
        IneligibleReason, NoPrimaryReason, PublicationGoal, ReconcileAction, ShutdownMode,
        SwitchoverBlocker, TargetRole,
    },
    pginfo::state::{Readiness, SqlStatus},
    state::Versioned,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {
    #[serde(default)]
    pub(crate) switchover_to: Option<String>,
}

pub(crate) fn post_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
    snapshot: Option<&SystemSnapshot>,
    input: SwitchoverRequestInput,
) -> ApiResult<AcceptedResponse> {
    let request = validate_switchover_request(snapshot, input)?;
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

pub(crate) fn get_ha_state(snapshot: &Versioned<SystemSnapshot>) -> HaStateResponse {
    HaStateResponse {
        cluster_name: snapshot.value.config.value.cluster.name.clone(),
        scope: snapshot.value.config.value.dcs.scope.clone(),
        self_member_id: snapshot.value.config.value.cluster.member_id.clone(),
        leader: snapshot
            .value
            .dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone()),
        switchover_pending: snapshot.value.dcs.value.cache.switchover.is_some(),
        switchover_to: snapshot
            .value
            .dcs
            .value
            .cache
            .switchover
            .as_ref()
            .and_then(|request| {
                request
                    .switchover_to
                    .as_ref()
                    .map(|member_id| member_id.0.clone())
            }),
        member_count: snapshot.value.dcs.value.cache.members.len(),
        members: snapshot
            .value
            .dcs
            .value
            .cache
            .members
            .values()
            .map(map_member_record)
            .collect(),
        dcs_trust: map_dcs_trust(&snapshot.value.dcs.value.trust),
        authority: map_authority(&snapshot.value.ha.value.publication.authority),
        fence_cutoff: snapshot
            .value
            .ha
            .value
            .publication
            .fence_cutoff
            .as_ref()
            .map(map_fence_cutoff),
        ha_role: map_target_role(&snapshot.value.ha.value.role),
        ha_tick: snapshot.value.ha.value.tick,
        planned_actions: snapshot
            .value
            .ha
            .value
            .planned_actions
            .iter()
            .map(map_action)
            .collect(),
        snapshot_sequence: snapshot.value.sequence,
    }
}

fn validate_switchover_request(
    snapshot: Option<&SystemSnapshot>,
    input: SwitchoverRequestInput,
) -> ApiResult<SwitchoverRequest> {
    let snapshot =
        snapshot.ok_or_else(|| ApiError::DcsStore("snapshot unavailable".to_string()))?;
    validate_switchover_source(snapshot)?;

    let Some(raw_target) = input.switchover_to else {
        return Ok(SwitchoverRequest {
            switchover_to: None,
        });
    };

    let target = raw_target.trim();
    if target.is_empty() {
        return Err(ApiError::bad_request(
            "switchover_to must not be empty".to_string(),
        ));
    }

    let target_member_id = crate::state::MemberId(target.to_string());
    let members = &snapshot.dcs.value.cache.members;
    let target_member = members
        .get(&target_member_id)
        .ok_or_else(|| ApiError::bad_request(format!("unknown switchover_to member `{target}`")))?;
    if target_member.role == MemberRole::Unknown
        || target_member.sql != SqlStatus::Healthy
        || target_member.readiness != Readiness::Ready
    {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is not an eligible switchover target"
        )));
    }

    if snapshot
        .dcs
        .value
        .cache
        .leader
        .as_ref()
        .map(|leader| leader.member_id == target_member_id)
        .unwrap_or(false)
    {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is already the leader"
        )));
    }

    Ok(SwitchoverRequest {
        switchover_to: Some(target_member_id),
    })
}

fn validate_switchover_source(snapshot: &SystemSnapshot) -> ApiResult<()> {
    if snapshot.dcs.value.trust != DcsTrust::FullQuorum {
        return Err(ApiError::bad_request(
            "switchover requests require full quorum DCS trust".to_string(),
        ));
    }

    let self_member_id = crate::state::MemberId(snapshot.config.value.cluster.member_id.clone());
    match &snapshot.ha.value.publication.authority {
        crate::ha::types::AuthorityView::Primary { member, .. } if *member == self_member_id => {
            Ok(())
        }
        _ => Err(ApiError::bad_request(
            "switchover requests must be sent to the authoritative primary".to_string(),
        )),
    }
}

fn map_dcs_trust(value: &DcsTrust) -> DcsTrustResponse {
    match value {
        DcsTrust::FullQuorum => DcsTrustResponse::FullQuorum,
        DcsTrust::FailSafe => DcsTrustResponse::FailSafe,
        DcsTrust::NotTrusted => DcsTrustResponse::NotTrusted,
    }
}

fn map_member_record(value: &MemberRecord) -> HaClusterMemberResponse {
    HaClusterMemberResponse {
        member_id: value.member_id.0.clone(),
        postgres_host: value.postgres_host.clone(),
        postgres_port: value.postgres_port,
        api_url: value.api_url.clone(),
        role: map_member_role(&value.role),
        sql: map_sql_status(&value.sql),
        readiness: map_readiness(&value.readiness),
        timeline: value.timeline.map(|timeline| u64::from(timeline.0)),
        write_lsn: value.write_lsn.map(|lsn| lsn.0),
        replay_lsn: value.replay_lsn.map(|lsn| lsn.0),
        pg_version: value.pg_version.0,
    }
}

fn map_authority(value: &crate::ha::types::AuthorityView) -> HaAuthorityResponse {
    match value {
        crate::ha::types::AuthorityView::Primary { member, epoch } => {
            HaAuthorityResponse::Primary {
                member_id: member.0.clone(),
                epoch: map_epoch(epoch),
            }
        }
        crate::ha::types::AuthorityView::NoPrimary(reason) => HaAuthorityResponse::NoPrimary {
            reason: map_no_primary_reason(reason),
        },
        crate::ha::types::AuthorityView::Unknown => HaAuthorityResponse::Unknown,
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

fn map_target_role(value: &TargetRole) -> TargetRoleResponse {
    match value {
        TargetRole::Leader(epoch) => TargetRoleResponse::Leader {
            epoch: map_epoch(epoch),
        },
        TargetRole::Candidate(candidacy) => TargetRoleResponse::Candidate {
            candidacy: map_candidacy(candidacy),
        },
        TargetRole::Follower(goal) => TargetRoleResponse::Follower {
            goal: map_follow_goal(goal),
        },
        TargetRole::FailSafe(goal) => TargetRoleResponse::FailSafe {
            goal: map_failsafe_goal(goal),
        },
        TargetRole::DemotingForSwitchover(member_id) => TargetRoleResponse::DemotingForSwitchover {
            member_id: member_id.0.clone(),
        },
        TargetRole::Fenced(reason) => TargetRoleResponse::Fenced {
            reason: map_fence_reason(reason),
        },
        TargetRole::Idle(reason) => TargetRoleResponse::Idle {
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

fn map_failsafe_goal(value: &FailSafeGoal) -> FailSafeGoalResponse {
    match value {
        FailSafeGoal::PrimaryMustStop(cutoff) => FailSafeGoalResponse::PrimaryMustStop {
            cutoff: map_fence_cutoff(cutoff),
        },
        FailSafeGoal::ReplicaKeepFollowing(upstream) => {
            FailSafeGoalResponse::ReplicaKeepFollowing {
                upstream: upstream.as_ref().map(|member_id| member_id.0.clone()),
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

fn map_action(value: &ReconcileAction) -> ReconcileActionResponse {
    match value {
        ReconcileAction::InitDb => ReconcileActionResponse::InitDb,
        ReconcileAction::BaseBackup(member_id) => ReconcileActionResponse::BaseBackup {
            member_id: member_id.0.clone(),
        },
        ReconcileAction::PgRewind(member_id) => ReconcileActionResponse::PgRewind {
            member_id: member_id.0.clone(),
        },
        ReconcileAction::StartPrimary => ReconcileActionResponse::StartPrimary,
        ReconcileAction::StartReplica(member_id) => ReconcileActionResponse::StartReplica {
            member_id: member_id.0.clone(),
        },
        ReconcileAction::Promote => ReconcileActionResponse::Promote,
        ReconcileAction::Demote(mode) => ReconcileActionResponse::Demote {
            mode: map_shutdown_mode(*mode),
        },
        ReconcileAction::AcquireLease(candidacy) => ReconcileActionResponse::AcquireLease {
            candidacy: map_candidacy(candidacy),
        },
        ReconcileAction::ReleaseLease => ReconcileActionResponse::ReleaseLease,
        ReconcileAction::EnsureRequiredRoles => ReconcileActionResponse::EnsureRequiredRoles,
        ReconcileAction::Publish(publication) => ReconcileActionResponse::Publish {
            publication: map_publication_goal(publication),
        },
        ReconcileAction::ClearSwitchover => ReconcileActionResponse::ClearSwitchover,
    }
}

fn map_publication_goal(value: &PublicationGoal) -> HaAuthorityResponse {
    match value {
        PublicationGoal::KeepCurrent => HaAuthorityResponse::Unknown,
        PublicationGoal::PublishPrimary { primary, epoch } => HaAuthorityResponse::Primary {
            member_id: primary.0.clone(),
            epoch: map_epoch(epoch),
        },
        PublicationGoal::PublishNoPrimary { reason, .. } => HaAuthorityResponse::NoPrimary {
            reason: map_no_primary_reason(reason),
        },
    }
}

fn map_shutdown_mode(value: ShutdownMode) -> ShutdownModeResponse {
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

fn map_member_role(value: &MemberRole) -> MemberRoleResponse {
    match value {
        MemberRole::Unknown => MemberRoleResponse::Unknown,
        MemberRole::Primary => MemberRoleResponse::Primary,
        MemberRole::Replica => MemberRoleResponse::Replica,
    }
}

fn map_sql_status(value: &crate::pginfo::state::SqlStatus) -> SqlStatusResponse {
    match value {
        crate::pginfo::state::SqlStatus::Unknown => SqlStatusResponse::Unknown,
        crate::pginfo::state::SqlStatus::Healthy => SqlStatusResponse::Healthy,
        crate::pginfo::state::SqlStatus::Unreachable => SqlStatusResponse::Unreachable,
    }
}

fn map_readiness(value: &crate::pginfo::state::Readiness) -> ReadinessResponse {
    match value {
        crate::pginfo::state::Readiness::Unknown => ReadinessResponse::Unknown,
        crate::pginfo::state::Readiness::Ready => ReadinessResponse::Ready,
        crate::pginfo::state::Readiness::NotReady => ReadinessResponse::NotReady,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        api::ApiError,
        config::RuntimeConfig,
        dcs::state::{DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole},
        debug_api::snapshot::{AppLifecycle, SystemSnapshot},
        ha::{
            state::HaState,
            types::{AuthorityView, LeaseEpoch, PublicationState, TargetRole},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{MemberId, UnixMillis, Version, Versioned, WorkerStatus},
    };

    use super::{validate_switchover_request, SwitchoverRequestInput};

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_member(
        member_id: &str,
        role: MemberRole,
        sql: SqlStatus,
        readiness: Readiness,
    ) -> MemberRecord {
        MemberRecord {
            member_id: MemberId(member_id.to_string()),
            postgres_host: member_id.to_string(),
            postgres_port: 5432,
            api_url: Some(format!("https://{member_id}:8443")),
            role,
            sql,
            readiness,
            timeline: Some(crate::state::TimelineId(1)),
            write_lsn: Some(crate::state::WalLsn(10)),
            replay_lsn: Some(crate::state::WalLsn(10)),
            pg_version: Version(1),
        }
    }

    fn sample_snapshot(target_member: MemberRecord) -> SystemSnapshot {
        let config = sample_runtime_config();
        let self_id = MemberId("node-a".to_string());
        let self_epoch = LeaseEpoch {
            holder: self_id.clone(),
            generation: 7,
        };
        let self_member = sample_member(
            "node-a",
            MemberRole::Primary,
            SqlStatus::Healthy,
            Readiness::Ready,
        );
        let members = BTreeMap::from([
            (self_member.member_id.clone(), self_member),
            (target_member.member_id.clone(), target_member),
        ]);

        SystemSnapshot {
            app: AppLifecycle::Running,
            config: Versioned::new(Version(1), UnixMillis(100), config.clone()),
            pg: Versioned::new(
                Version(1),
                UnixMillis(100),
                PgInfoState::Unknown {
                    common: PgInfoCommon {
                        worker: WorkerStatus::Running,
                        sql: SqlStatus::Healthy,
                        readiness: Readiness::Ready,
                        timeline: None,
                        pg_config: PgConfig {
                            port: Some(5432),
                            hot_standby: Some(false),
                            primary_conninfo: None,
                            primary_slot_name: None,
                            extra: BTreeMap::new(),
                        },
                        last_refresh_at: Some(UnixMillis(100)),
                    },
                },
            ),
            dcs: Versioned::new(
                Version(1),
                UnixMillis(100),
                DcsState {
                    worker: WorkerStatus::Running,
                    trust: DcsTrust::FullQuorum,
                    cache: DcsCache {
                        members,
                        leader: Some(LeaderRecord {
                            member_id: self_id.clone(),
                            generation: self_epoch.generation,
                        }),
                        switchover: None,
                        config,
                        init_lock: None,
                    },
                    last_refresh_at: Some(UnixMillis(100)),
                },
            ),
            process: Versioned::new(
                Version(1),
                UnixMillis(100),
                ProcessState::Idle {
                    worker: WorkerStatus::Running,
                    last_outcome: None,
                },
            ),
            ha: Versioned::new(
                Version(1),
                UnixMillis(100),
                HaState {
                    worker: WorkerStatus::Running,
                    tick: 7,
                    required_roles_ready: false,
                    publication: PublicationState {
                        authority: AuthorityView::Primary {
                            member: self_id.clone(),
                            epoch: self_epoch.clone(),
                        },
                        fence_cutoff: None,
                    },
                    role: TargetRole::Leader(self_epoch),
                    clear_switchover: false,
                    planned_actions: Vec::new(),
                },
            ),
            generated_at: UnixMillis(100),
            sequence: 1,
            changes: Vec::new(),
            timeline: Vec::new(),
        }
    }

    #[test]
    fn validate_switchover_request_rejects_ineligible_target() {
        let snapshot = sample_snapshot(sample_member(
            "node-c",
            MemberRole::Unknown,
            SqlStatus::Unknown,
            Readiness::Unknown,
        ));

        let result = validate_switchover_request(
            Some(&snapshot),
            SwitchoverRequestInput {
                switchover_to: Some("node-c".to_string()),
            },
        );

        assert_eq!(
            result,
            Err(ApiError::bad_request(
                "switchover_to member `node-c` is not an eligible switchover target".to_string()
            ))
        );
    }

    #[test]
    fn validate_switchover_request_rejects_unknown_target() {
        let snapshot = sample_snapshot(sample_member(
            "node-c",
            MemberRole::Replica,
            SqlStatus::Healthy,
            Readiness::Ready,
        ));

        let result = validate_switchover_request(
            Some(&snapshot),
            SwitchoverRequestInput {
                switchover_to: Some("node-z".to_string()),
            },
        );

        assert_eq!(
            result,
            Err(ApiError::bad_request(
                "unknown switchover_to member `node-z`".to_string()
            ))
        );
    }

    #[test]
    fn validate_switchover_request_rejects_current_leader_target() {
        let snapshot = sample_snapshot(sample_member(
            "node-c",
            MemberRole::Replica,
            SqlStatus::Healthy,
            Readiness::Ready,
        ));

        let result = validate_switchover_request(
            Some(&snapshot),
            SwitchoverRequestInput {
                switchover_to: Some("node-a".to_string()),
            },
        );

        assert_eq!(
            result,
            Err(ApiError::bad_request(
                "switchover_to member `node-a` is already the leader".to_string()
            ))
        );
    }

    #[test]
    fn validate_switchover_request_accepts_healthy_non_leader_target() {
        let snapshot = sample_snapshot(sample_member(
            "node-c",
            MemberRole::Replica,
            SqlStatus::Healthy,
            Readiness::Ready,
        ));

        let result = validate_switchover_request(
            Some(&snapshot),
            SwitchoverRequestInput {
                switchover_to: Some("node-c".to_string()),
            },
        );

        assert_eq!(
            result,
            Ok(crate::dcs::state::SwitchoverRequest {
                switchover_to: Some(MemberId("node-c".to_string())),
            })
        );
    }
}
