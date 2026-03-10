use serde::{Deserialize, Serialize};

use crate::{
    api::{
        AcceptedResponse, ApiError, ApiResult, BootstrapPlanResponse, ClusterModeResponse,
        DcsTrustResponse, DesiredNodeStateResponse, FencePlanResponse, HaClusterMemberResponse,
        HaStateResponse, MemberRoleResponse, PrimaryPlanResponse, QuiescentReasonResponse,
        ReadinessResponse, ReplicaPlanResponse, SqlStatusResponse,
    },
    dcs::{
        state::{member_record_is_fresh, DcsTrust, MemberRecord, MemberRole, SwitchoverRequest},
        store::DcsStore,
    },
    debug_api::snapshot::SystemSnapshot,
    ha::{
        decision::eligible_switchover_targets,
        state::{
            BootstrapPlan, ClusterMode, DesiredNodeState, FencePlan, PrimaryPlan, QuiescentReason,
            ReplicaPlan,
        },
    },
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
            .and_then(|request| request.switchover_to.as_ref().map(|member_id| member_id.0.clone())),
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
        cluster_mode: map_cluster_mode(&snapshot.value.ha.value.cluster_mode),
        desired_state: map_desired_state(&snapshot.value.ha.value.desired_state),
        ha_tick: snapshot.value.ha.value.tick,
        snapshot_sequence: snapshot.value.sequence,
    }
}

fn validate_switchover_request(
    snapshot: Option<&SystemSnapshot>,
    input: SwitchoverRequestInput,
) -> ApiResult<SwitchoverRequest> {
    let Some(raw_target) = input.switchover_to else {
        return Ok(SwitchoverRequest { switchover_to: None });
    };
    let snapshot =
        snapshot.ok_or_else(|| ApiError::DcsStore("snapshot unavailable".to_string()))?;

    let target = raw_target.trim();
    if target.is_empty() {
        return Err(ApiError::bad_request("switchover_to must not be empty".to_string()));
    }

    let target_member_id = crate::state::MemberId(target.to_string());
    let members = &snapshot.dcs.value.cache.members;
    let target_member = members
        .get(&target_member_id)
        .ok_or_else(|| ApiError::bad_request(format!("unknown switchover_to member `{target}`")))?;

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

    let now = crate::process::worker::system_now_unix_millis()
        .map_err(|err| ApiError::internal(format!("switchover current-time read failed: {err}")))?;
    if !member_record_is_fresh(target_member, &snapshot.dcs.value.cache, now) {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is not an eligible switchover target"
        )));
    }

    let eligible_targets = eligible_switchover_targets(&crate::ha::state::WorldSnapshot {
        config: snapshot.config.clone(),
        pg: snapshot.pg.clone(),
        dcs: snapshot.dcs.clone(),
        process: snapshot.process.clone(),
    });
    if !eligible_targets.contains(&target_member_id) {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is not an eligible switchover target"
        )));
    }

    Ok(SwitchoverRequest {
        switchover_to: Some(target_member_id),
    })
}

fn map_dcs_trust(value: &DcsTrust) -> DcsTrustResponse {
    match value {
        DcsTrust::FreshQuorum => DcsTrustResponse::FreshQuorum,
        DcsTrust::NoFreshQuorum => DcsTrustResponse::NoFreshQuorum,
        DcsTrust::NotTrusted => DcsTrustResponse::NotTrusted,
    }
}

fn map_cluster_mode(value: &ClusterMode) -> ClusterModeResponse {
    match value {
        ClusterMode::DcsUnavailable => ClusterModeResponse::DcsUnavailable,
        ClusterMode::UninitializedNoBootstrapOwner => ClusterModeResponse::UninitializedNoBootstrapOwner,
        ClusterMode::UninitializedBootstrapInProgress { holder } => {
            ClusterModeResponse::UninitializedBootstrapInProgress {
                holder: holder.0.clone(),
            }
        }
        ClusterMode::InitializedLeaderPresent { leader } => {
            ClusterModeResponse::InitializedLeaderPresent {
                leader: leader.0.clone(),
            }
        }
        ClusterMode::InitializedNoLeaderFreshQuorum => ClusterModeResponse::InitializedNoLeaderFreshQuorum,
        ClusterMode::InitializedNoLeaderNoFreshQuorum => {
            ClusterModeResponse::InitializedNoLeaderNoFreshQuorum
        }
    }
}

fn map_desired_state(value: &DesiredNodeState) -> DesiredNodeStateResponse {
    match value {
        DesiredNodeState::Bootstrap { plan } => DesiredNodeStateResponse::Bootstrap {
            plan: match plan {
                BootstrapPlan::InitDb => BootstrapPlanResponse::InitDb,
            },
        },
        DesiredNodeState::Primary { plan } => DesiredNodeStateResponse::Primary {
            plan: match plan {
                PrimaryPlan::KeepLeader => PrimaryPlanResponse::KeepLeader,
                PrimaryPlan::AcquireLeaderThenResumePrimary => {
                    PrimaryPlanResponse::AcquireLeaderThenResumePrimary
                }
                PrimaryPlan::AcquireLeaderThenPromote => {
                    PrimaryPlanResponse::AcquireLeaderThenPromote
                }
                PrimaryPlan::AcquireLeaderThenStartPrimary => {
                    PrimaryPlanResponse::AcquireLeaderThenStartPrimary
                }
            },
        },
        DesiredNodeState::Replica { plan } => DesiredNodeStateResponse::Replica {
            plan: match plan {
                ReplicaPlan::DirectFollow { leader_member_id } => {
                    ReplicaPlanResponse::DirectFollow {
                        leader_member_id: leader_member_id.0.clone(),
                    }
                }
                ReplicaPlan::RewindThenFollow { leader_member_id } => {
                    ReplicaPlanResponse::RewindThenFollow {
                        leader_member_id: leader_member_id.0.clone(),
                    }
                }
                ReplicaPlan::BasebackupThenFollow { leader_member_id } => {
                    ReplicaPlanResponse::BasebackupThenFollow {
                        leader_member_id: leader_member_id.0.clone(),
                    }
                }
            },
        },
        DesiredNodeState::Quiescent { reason } => DesiredNodeStateResponse::Quiescent {
            reason: match reason {
                QuiescentReason::WaitingForBootstrapWinner => {
                    QuiescentReasonResponse::WaitingForBootstrapWinner
                }
                QuiescentReason::WaitingForAuthoritativeLeader => {
                    QuiescentReasonResponse::WaitingForAuthoritativeLeader
                }
                QuiescentReason::WaitingForFreshQuorum => {
                    QuiescentReasonResponse::WaitingForFreshQuorum
                }
                QuiescentReason::WaitingForAuthoritativeClusterState => {
                    QuiescentReasonResponse::WaitingForAuthoritativeClusterState
                }
                QuiescentReason::WaitingForRecoveryPreconditions => {
                    QuiescentReasonResponse::WaitingForRecoveryPreconditions
                }
                QuiescentReason::UnsafeUninitializedPgData => {
                    QuiescentReasonResponse::UnsafeUninitializedPgData
                }
            },
        },
        DesiredNodeState::Fence { plan } => DesiredNodeStateResponse::Fence {
            plan: match plan {
                FencePlan::StopAndStayNonWritable => FencePlanResponse::StopAndStayNonWritable,
            },
        },
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
        updated_at_ms: value.updated_at.0,
        pg_version: value.pg_version.0,
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
