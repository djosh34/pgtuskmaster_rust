use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    time::Duration,
};

use reqwest::Url;
use serde::Serialize;
use tokio::task::JoinSet;

use crate::{
    api::{HaClusterMemberResponse, HaDecisionResponse, HaStateResponse},
    cli::{
        args::StatusOptions,
        client::{CliApiClient, CliApiClientConfig, DebugVerboseResponse},
        config::OperatorContext,
        error::CliError,
        output,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterHealth {
    Healthy,
    Degraded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiStatus {
    Ok,
    Down,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugObservationStatus {
    Available,
    Disabled,
    AuthFailed,
    NotReady,
    TransportFailed,
    DecodeFailed,
    ApiStatusFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct QueryOrigin {
    pub member_id: String,
    pub api_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterSwitchoverView {
    pub pending: bool,
    pub target_member_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterNodeDebugObservation {
    pub status: DebugObservationStatus,
    pub detail: Option<String>,
    pub payload: Option<DebugVerboseResponse>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterNodeView {
    pub member_id: String,
    pub is_self: bool,
    pub sampled: bool,
    pub api_url: Option<String>,
    pub api_status: ApiStatus,
    pub role: String,
    pub trust: String,
    pub phase: String,
    pub leader: Option<String>,
    pub decision: Option<String>,
    pub pginfo: Option<String>,
    pub readiness: Option<String>,
    pub process: Option<String>,
    pub debug: Option<ClusterNodeDebugObservation>,
    pub observation_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterStatusView {
    pub cluster_name: String,
    pub scope: String,
    pub verbose: bool,
    pub queried_via: QueryOrigin,
    pub sampled_member_count: usize,
    pub discovered_member_count: usize,
    pub health: ClusterHealth,
    pub warnings: Vec<ClusterWarning>,
    pub switchover: Option<ClusterSwitchoverView>,
    pub nodes: Vec<ClusterNodeView>,
}

#[derive(Clone, Debug)]
pub(crate) struct SampledNodeState {
    pub(crate) state: HaStateResponse,
    pub(crate) debug: Option<ClusterNodeDebugObservation>,
}

#[derive(Clone, Debug)]
pub(crate) struct PeerObservation {
    pub(crate) member_id: String,
    pub(crate) sampled: Result<SampledNodeState, String>,
}

#[derive(Clone, Debug)]
pub(crate) struct SampledClusterSnapshot {
    pub(crate) seed_state: HaStateResponse,
    pub(crate) discovered_members: Vec<HaClusterMemberResponse>,
    pub(crate) queried_via: QueryOrigin,
    pub(crate) observations: BTreeMap<String, PeerObservation>,
    pub(crate) warnings: Vec<ClusterWarning>,
}

impl SampledClusterSnapshot {
    pub(crate) fn sampled_member_count(&self) -> usize {
        self.observations
            .values()
            .filter(|value| value.sampled.is_ok())
            .count()
    }

    pub(crate) fn discovered_member_count(&self) -> usize {
        self.discovered_members.len()
    }
}

pub(crate) async fn run_status(
    context: &OperatorContext,
    options: StatusOptions,
) -> Result<String, CliError> {
    if options.watch {
        run_watch(context, options).await
    } else {
        let view = build_cluster_status_view(context, options).await?;
        output::render_status_view(&view, options.json)
    }
}

pub(crate) async fn build_cluster_status_view(
    context: &OperatorContext,
    options: StatusOptions,
) -> Result<ClusterStatusView, CliError> {
    let snapshot = build_sampled_cluster_snapshot(context, options.verbose).await?;
    Ok(assemble_cluster_view(&snapshot, options.verbose))
}

pub(crate) async fn build_sampled_cluster_snapshot(
    context: &OperatorContext,
    verbose: bool,
) -> Result<SampledClusterSnapshot, CliError> {
    let seed_client = CliApiClient::from_config(context.api_client.clone())?;
    let seed_state = seed_client.get_ha_state().await?;
    let seed_debug = fetch_debug_observation(&seed_client, verbose).await;
    let discovered_members = seed_state.members.clone();
    let queried_via = QueryOrigin {
        member_id: seed_state.self_member_id.clone(),
        api_url: seed_client.base_url().to_string(),
    };

    let peer_observations =
        sample_peer_states(&context.api_client, &seed_state, seed_debug, verbose).await;
    let warnings = collect_warnings(&seed_state, &discovered_members, &peer_observations);

    Ok(SampledClusterSnapshot {
        seed_state,
        discovered_members,
        queried_via,
        observations: peer_observations,
        warnings,
    })
}

async fn run_watch(context: &OperatorContext, options: StatusOptions) -> Result<String, CliError> {
    let mut stdout = std::io::stdout();
    let interval = Duration::from_secs(2);

    loop {
        let view = build_cluster_status_view(context, options).await?;
        let rendered = output::render_status_view(&view, options.json)?;
        if options.json {
            writeln!(stdout, "{rendered}")
                .map_err(|err| CliError::Output(format!("watch write failed: {err}")))?;
        } else {
            writeln!(stdout, "\x1B[2J\x1B[H{rendered}")
                .map_err(|err| CliError::Output(format!("watch write failed: {err}")))?;
        }
        stdout
            .flush()
            .map_err(|err| CliError::Output(format!("watch flush failed: {err}")))?;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(String::new()),
            _ = tokio::time::sleep(interval) => {}
        }
    }
}

async fn sample_peer_states(
    base_config: &CliApiClientConfig,
    seed_state: &HaStateResponse,
    seed_debug: Option<ClusterNodeDebugObservation>,
    verbose: bool,
) -> BTreeMap<String, PeerObservation> {
    let seed_observation = PeerObservation {
        member_id: seed_state.self_member_id.clone(),
        sampled: Ok(SampledNodeState {
            state: seed_state.clone(),
            debug: seed_debug,
        }),
    };

    let mut observations = BTreeMap::from([(seed_state.self_member_id.clone(), seed_observation)]);
    let mut join_set = JoinSet::new();

    for member in &seed_state.members {
        if member.member_id == seed_state.self_member_id {
            continue;
        }

        let Some(api_url) = member.api_url.as_deref() else {
            observations.insert(
                member.member_id.clone(),
                PeerObservation {
                    member_id: member.member_id.clone(),
                    sampled: Err("missing advertised api_url".to_string()),
                },
            );
            continue;
        };

        let parsed_url = match Url::parse(api_url) {
            Ok(value) => value,
            Err(err) => {
                observations.insert(
                    member.member_id.clone(),
                    PeerObservation {
                        member_id: member.member_id.clone(),
                        sampled: Err(format!("invalid advertised api_url `{api_url}`: {err}")),
                    },
                );
                continue;
            }
        };

        let config = base_config.with_base_url(parsed_url);
        let member_id = member.member_id.clone();
        join_set.spawn(async move {
            match CliApiClient::from_config(config) {
                Ok(client) => match client.get_ha_state().await {
                    Ok(state) => {
                        let debug_observation = fetch_debug_observation(&client, verbose).await;
                        PeerObservation {
                            member_id: member_id.clone(),
                            sampled: Ok(SampledNodeState {
                                state,
                                debug: debug_observation,
                            }),
                        }
                    }
                    Err(err) => PeerObservation {
                        member_id: member_id.clone(),
                        sampled: Err(err.to_string()),
                    },
                },
                Err(err) => PeerObservation {
                    member_id: member_id.clone(),
                    sampled: Err(err.to_string()),
                },
            }
        });
    }

    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(observation) => {
                observations.insert(observation.member_id.clone(), observation);
            }
            Err(err) => {
                let member_id = format!("task-join-error-{}", observations.len());
                observations.insert(
                    member_id.clone(),
                    PeerObservation {
                        member_id,
                        sampled: Err(format!("peer sampling task failed: {err}")),
                    },
                );
            }
        }
    }

    observations
}

async fn fetch_debug_observation(
    client: &CliApiClient,
    verbose: bool,
) -> Option<ClusterNodeDebugObservation> {
    if !verbose {
        return None;
    }

    match client.get_debug_verbose().await {
        Ok(value) => Some(ClusterNodeDebugObservation {
            status: DebugObservationStatus::Available,
            detail: None,
            payload: Some(value),
        }),
        Err(CliError::ApiStatus { status, body }) if status == 404 => {
            Some(ClusterNodeDebugObservation {
                status: DebugObservationStatus::Disabled,
                detail: summarize_api_failure(status, body.as_str()),
                payload: None,
            })
        }
        Err(CliError::ApiStatus { status, body }) if status == 401 || status == 403 => {
            Some(ClusterNodeDebugObservation {
                status: DebugObservationStatus::AuthFailed,
                detail: summarize_api_failure(status, body.as_str()),
                payload: None,
            })
        }
        Err(CliError::ApiStatus { status, body }) if status == 503 => {
            Some(ClusterNodeDebugObservation {
                status: DebugObservationStatus::NotReady,
                detail: summarize_api_failure(status, body.as_str()),
                payload: None,
            })
        }
        Err(CliError::ApiStatus { status, body }) => Some(ClusterNodeDebugObservation {
            status: DebugObservationStatus::ApiStatusFailed,
            detail: summarize_api_failure(status, body.as_str()),
            payload: None,
        }),
        Err(CliError::Transport(message) | CliError::RequestBuild(message)) => {
            Some(ClusterNodeDebugObservation {
                status: DebugObservationStatus::TransportFailed,
                detail: Some(message),
                payload: None,
            })
        }
        Err(CliError::Decode(message)) => Some(ClusterNodeDebugObservation {
            status: DebugObservationStatus::DecodeFailed,
            detail: Some(message),
            payload: None,
        }),
        Err(
            CliError::Config(message) | CliError::Resolution(message) | CliError::Output(message),
        ) => Some(ClusterNodeDebugObservation {
            status: DebugObservationStatus::ApiStatusFailed,
            detail: Some(message),
            payload: None,
        }),
    }
}

fn summarize_api_failure(status: u16, body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Some(format!("http {status}"));
    }

    let first_line = match trimmed.lines().next() {
        Some(value) => value,
        None => trimmed,
    };
    let clipped = if first_line.chars().count() > 120 {
        let shortened = first_line.chars().take(120).collect::<String>();
        format!("{shortened}...")
    } else {
        first_line.to_string()
    };
    Some(format!("http {status}: {clipped}"))
}

fn assemble_cluster_view(snapshot: &SampledClusterSnapshot, verbose: bool) -> ClusterStatusView {
    let mut nodes = snapshot
        .discovered_members
        .iter()
        .map(|member| {
            build_node_row(
                member,
                snapshot.queried_via.member_id.as_str(),
                snapshot.observations.get(&member.member_id),
            )
        })
        .collect::<Vec<_>>();
    nodes.sort_by(node_sort_key);

    ClusterStatusView {
        cluster_name: snapshot.seed_state.cluster_name.clone(),
        scope: snapshot.seed_state.scope.clone(),
        verbose,
        queried_via: snapshot.queried_via.clone(),
        sampled_member_count: snapshot.sampled_member_count(),
        discovered_member_count: snapshot.discovered_member_count(),
        health: if snapshot.warnings.is_empty() {
            ClusterHealth::Healthy
        } else {
            ClusterHealth::Degraded
        },
        warnings: snapshot.warnings.clone(),
        switchover: snapshot
            .seed_state
            .switchover_pending
            .then_some(ClusterSwitchoverView {
                pending: true,
                target_member_id: snapshot.seed_state.switchover_to.clone(),
            }),
        nodes,
    }
}

fn collect_warnings(
    seed_state: &HaStateResponse,
    discovered_members: &[HaClusterMemberResponse],
    observations: &BTreeMap<String, PeerObservation>,
) -> Vec<ClusterWarning> {
    let mut warnings = Vec::new();
    let mut sampled_leaders = BTreeSet::new();
    let mut sampled_primary_members = BTreeSet::new();
    let seed_members = discovered_members
        .iter()
        .map(|member| member.member_id.clone())
        .collect::<BTreeSet<_>>();

    for member in discovered_members {
        let observation = observations.get(&member.member_id);
        match observation {
            Some(PeerObservation {
                sampled: Ok(sampled),
                ..
            }) => {
                sampled_leaders.insert(
                    sampled
                        .state
                        .leader
                        .clone()
                        .unwrap_or_else(|| "<none>".to_string()),
                );
                if local_role_from_state(&sampled.state) == "primary" {
                    sampled_primary_members.insert(sampled.state.self_member_id.clone());
                }
                if sampled.state.dcs_trust != crate::api::DcsTrustResponse::FullQuorum {
                    warnings.push(ClusterWarning {
                        code: "degraded_trust".to_string(),
                        message: format!(
                            "node {} reports trust {}",
                            sampled.state.self_member_id, sampled.state.dcs_trust
                        ),
                    });
                }
                let sampled_members = sampled
                    .state
                    .members
                    .iter()
                    .map(|value| value.member_id.clone())
                    .collect::<BTreeSet<_>>();
                if sampled_members != seed_members {
                    warnings.push(ClusterWarning {
                        code: "membership_mismatch".to_string(),
                        message: format!(
                            "node {} reports a different member set than queried_via {}",
                            sampled.state.self_member_id, seed_state.self_member_id
                        ),
                    });
                }
            }
            Some(PeerObservation {
                sampled: Err(message),
                ..
            }) => warnings.push(ClusterWarning {
                code: if member.api_url.is_some() {
                    "unreachable_node".to_string()
                } else {
                    "missing_api_url".to_string()
                },
                message: format!("node {} could not be sampled: {message}", member.member_id),
            }),
            None => warnings.push(ClusterWarning {
                code: "missing_observation".to_string(),
                message: format!("node {} was not sampled", member.member_id),
            }),
        }
    }

    if sampled_leaders.len() > 1 {
        warnings.push(ClusterWarning {
            code: "leader_mismatch".to_string(),
            message: format!(
                "sampled nodes disagree on leader: {}",
                sampled_leaders.into_iter().collect::<Vec<_>>().join(", ")
            ),
        });
    }

    if sampled_primary_members.len() > 1 {
        warnings.push(ClusterWarning {
            code: "multi_primary".to_string(),
            message: format!(
                "multiple sampled primaries: {}",
                sampled_primary_members
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }

    let sampled_member_count = observations
        .values()
        .filter(|value| value.sampled.is_ok())
        .count();
    if sampled_member_count < discovered_members.len() {
        warnings.push(ClusterWarning {
            code: "insufficient_sampling".to_string(),
            message: format!(
                "sampled {sampled_member_count}/{} discovered members",
                discovered_members.len()
            ),
        });
    }

    warnings
}

fn build_node_row(
    member: &HaClusterMemberResponse,
    queried_member_id: &str,
    observation: Option<&PeerObservation>,
) -> ClusterNodeView {
    match observation {
        Some(PeerObservation {
            sampled: Ok(sampled),
            ..
        }) => {
            let debug_payload = sampled
                .debug
                .as_ref()
                .and_then(|observation| observation.payload.as_ref());
            ClusterNodeView {
                member_id: member.member_id.clone(),
                is_self: member.member_id == queried_member_id,
                sampled: true,
                api_url: member.api_url.clone(),
                api_status: ApiStatus::Ok,
                role: local_role_from_state(&sampled.state).to_string(),
                trust: sampled.state.dcs_trust.to_string(),
                phase: sampled.state.ha_phase.to_string(),
                leader: sampled.state.leader.clone(),
                decision: Some(render_decision_text(&sampled.state.ha_decision)),
                pginfo: debug_payload.map(|value| value.pginfo.summary.clone()),
                readiness: debug_payload.map(|value| value.pginfo.readiness.to_ascii_lowercase()),
                process: debug_payload.map(|value| value.process.state.to_ascii_lowercase()),
                debug: sampled.debug.clone(),
                observation_error: None,
            }
        }
        Some(PeerObservation {
            sampled: Err(message),
            ..
        }) => ClusterNodeView {
            member_id: member.member_id.clone(),
            is_self: member.member_id == queried_member_id,
            sampled: false,
            api_url: member.api_url.clone(),
            api_status: if member.api_url.is_some() {
                ApiStatus::Down
            } else {
                ApiStatus::Missing
            },
            role: "unknown".to_string(),
            trust: "unknown".to_string(),
            phase: "unknown".to_string(),
            leader: None,
            decision: None,
            pginfo: None,
            readiness: None,
            process: None,
            debug: None,
            observation_error: Some(message.clone()),
        },
        None => ClusterNodeView {
            member_id: member.member_id.clone(),
            is_self: member.member_id == queried_member_id,
            sampled: false,
            api_url: member.api_url.clone(),
            api_status: ApiStatus::Missing,
            role: "unknown".to_string(),
            trust: "unknown".to_string(),
            phase: "unknown".to_string(),
            leader: None,
            decision: None,
            pginfo: None,
            readiness: None,
            process: None,
            debug: None,
            observation_error: Some("no observation recorded".to_string()),
        },
    }
}

fn node_sort_key(left: &ClusterNodeView, right: &ClusterNodeView) -> std::cmp::Ordering {
    (
        !left.is_self,
        role_rank(left.role.as_str()),
        left.member_id.as_str(),
    )
        .cmp(&(
            !right.is_self,
            role_rank(right.role.as_str()),
            right.member_id.as_str(),
        ))
}

fn role_rank(role: &str) -> u8 {
    match role {
        "primary" => 0,
        "replica" => 1,
        _ => 2,
    }
}

pub(crate) fn local_role_from_state(state: &HaStateResponse) -> &'static str {
    match state.ha_phase {
        crate::api::HaPhaseResponse::Primary => "primary",
        crate::api::HaPhaseResponse::Replica => "replica",
        _ => "unknown",
    }
}

fn render_decision_text(value: &HaDecisionResponse) -> String {
    match value {
        HaDecisionResponse::NoChange => "no_change".to_string(),
        HaDecisionResponse::WaitForPostgres {
            start_requested,
            leader_member_id,
        } => {
            let leader_detail = leader_member_id.as_deref().unwrap_or("none");
            format!(
                "wait_for_postgres(start_requested={start_requested}, leader_member_id={leader_detail})"
            )
        }
        HaDecisionResponse::WaitForDcsTrust => "wait_for_dcs_trust".to_string(),
        HaDecisionResponse::WaitForPromotionSafety { blocker } => {
            format!("wait_for_promotion_safety(blocker={blocker})")
        }
        HaDecisionResponse::AttemptLeadership => "attempt_leadership".to_string(),
        HaDecisionResponse::FollowLeader { leader_member_id } => {
            format!("follow_leader(leader_member_id={leader_member_id})")
        }
        HaDecisionResponse::BecomePrimary { promote } => {
            format!("become_primary(promote={promote})")
        }
        HaDecisionResponse::CompleteSwitchover => "complete_switchover".to_string(),
        HaDecisionResponse::StepDown {
            reason,
            release_leader_lease,
            fence,
        } => format!(
            "step_down(reason={reason}, release_leader_lease={release_leader_lease}, fence={fence})"
        ),
        HaDecisionResponse::RecoverReplica { strategy } => {
            format!("recover_replica(strategy={strategy})")
        }
        HaDecisionResponse::FenceNode => "fence_node".to_string(),
        HaDecisionResponse::ReleaseLeaderLease { reason } => {
            format!("release_leader_lease(reason={reason})")
        }
        HaDecisionResponse::EnterFailSafe {
            release_leader_lease,
        } => format!("enter_fail_safe(release_leader_lease={release_leader_lease})"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        api::{
            DcsTrustResponse, HaClusterMemberResponse, HaDecisionResponse, HaPhaseResponse,
            HaStateResponse, MemberRoleResponse, ReadinessResponse, SqlStatusResponse,
        },
        cli::{
            client::DebugVerboseResponse,
            status::{
                assemble_cluster_view, ApiStatus, ClusterNodeDebugObservation,
                DebugObservationStatus, QueryOrigin, SampledClusterSnapshot,
            },
        },
        debug_api::view::{
            ApiSection, ConfigSection, DcsSection, DebugChangeView, DebugMeta, DebugSection,
            DebugTimelineView, HaSection, PgInfoSection, ProcessSection,
        },
    };

    fn sample_member(member_id: &str, api_url: Option<&str>) -> HaClusterMemberResponse {
        HaClusterMemberResponse {
            member_id: member_id.to_string(),
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            api_url: api_url.map(ToString::to_string),
            role: MemberRoleResponse::Replica,
            sql: SqlStatusResponse::Healthy,
            readiness: ReadinessResponse::Ready,
            timeline: Some(7),
            write_lsn: None,
            replay_lsn: Some(5),
            updated_at_ms: 1,
            pg_version: 1,
        }
    }

    fn sample_state(
        self_member_id: &str,
        phase: HaPhaseResponse,
        trust: DcsTrustResponse,
        leader: Option<&str>,
        members: Vec<HaClusterMemberResponse>,
    ) -> HaStateResponse {
        HaStateResponse {
            cluster_name: "cluster-a".to_string(),
            scope: "scope-a".to_string(),
            self_member_id: self_member_id.to_string(),
            leader: leader.map(ToString::to_string),
            switchover_pending: false,
            switchover_to: None,
            member_count: members.len(),
            members,
            dcs_trust: trust,
            ha_phase: phase,
            ha_tick: 1,
            ha_decision: HaDecisionResponse::NoChange,
            snapshot_sequence: 10,
        }
    }

    fn sample_snapshot(
        seed_state: HaStateResponse,
        discovered_members: Vec<HaClusterMemberResponse>,
        observations: BTreeMap<String, super::PeerObservation>,
    ) -> SampledClusterSnapshot {
        let warnings = super::collect_warnings(&seed_state, &discovered_members, &observations);
        SampledClusterSnapshot {
            seed_state,
            discovered_members,
            queried_via: QueryOrigin {
                member_id: "node-a".to_string(),
                api_url: "http://node-a:8080".to_string(),
            },
            observations,
            warnings,
        }
    }

    fn sample_debug_payload(member_id: &str) -> DebugVerboseResponse {
        DebugVerboseResponse {
            meta: DebugMeta {
                schema_version: "v1".to_string(),
                generated_at_ms: 1,
                channel_updated_at_ms: 1,
                channel_version: 1,
                app_lifecycle: "Running".to_string(),
                sequence: 42,
            },
            config: ConfigSection {
                version: 1,
                updated_at_ms: 1,
                cluster_name: "cluster-a".to_string(),
                member_id: member_id.to_string(),
                scope: "scope-a".to_string(),
                debug_enabled: true,
                tls_enabled: false,
            },
            pginfo: PgInfoSection {
                version: 1,
                updated_at_ms: 1,
                variant: "Primary".to_string(),
                worker: "Running".to_string(),
                sql: "Healthy".to_string(),
                readiness: "Ready".to_string(),
                timeline: Some(7),
                summary: "primary wal_lsn=7 readiness=Ready".to_string(),
            },
            dcs: DcsSection {
                version: 1,
                updated_at_ms: 1,
                worker: "Running".to_string(),
                trust: "FullQuorum".to_string(),
                member_count: 1,
                leader: Some("node-a".to_string()),
                has_switchover_request: false,
            },
            process: ProcessSection {
                version: 1,
                updated_at_ms: 1,
                worker: "Running".to_string(),
                state: "Idle".to_string(),
                running_job_id: None,
                last_outcome: Some("Success(job-1)".to_string()),
            },
            ha: HaSection {
                version: 1,
                updated_at_ms: 1,
                worker: "Running".to_string(),
                phase: "Primary".to_string(),
                tick: 1,
                decision: "NoChange".to_string(),
                decision_detail: Some("steady".to_string()),
                planned_actions: 0,
            },
            api: ApiSection {
                endpoints: vec!["/debug/verbose".to_string()],
            },
            debug: DebugSection {
                history_changes: 1,
                history_timeline: 1,
                last_sequence: 42,
            },
            changes: vec![DebugChangeView {
                sequence: 41,
                at_ms: 1,
                domain: "ha".to_string(),
                previous_version: Some(1),
                current_version: Some(2),
                summary: "decision updated".to_string(),
            }],
            timeline: vec![DebugTimelineView {
                sequence: 42,
                at_ms: 1,
                category: "ha".to_string(),
                message: "primary steady".to_string(),
            }],
        }
    }

    #[test]
    fn assemble_cluster_view_marks_missing_api_targets_as_degraded() {
        let members = vec![
            sample_member("node-a", Some("http://node-a:8080")),
            sample_member("node-b", None),
        ];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([
            (
                "node-a".to_string(),
                super::PeerObservation {
                    member_id: "node-a".to_string(),
                    sampled: Ok(super::SampledNodeState {
                        state: seed_state.clone(),
                        debug: None,
                    }),
                },
            ),
            (
                "node-b".to_string(),
                super::PeerObservation {
                    member_id: "node-b".to_string(),
                    sampled: Err("missing advertised api_url".to_string()),
                },
            ),
        ]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, false);

        assert_eq!(view.health, super::ClusterHealth::Degraded);
        assert!(view
            .warnings
            .iter()
            .any(|warning| warning.code == "missing_api_url"));
        assert!(view
            .nodes
            .iter()
            .any(|node| node.api_status == ApiStatus::Missing));
    }

    #[test]
    fn assemble_cluster_view_marks_multi_primary_as_degraded() {
        let members = vec![
            sample_member("node-a", Some("http://node-a:8080")),
            sample_member("node-b", Some("http://node-b:8080")),
        ];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let other_state = sample_state(
            "node-b",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-b"),
            members.clone(),
        );
        let observations = BTreeMap::from([
            (
                "node-a".to_string(),
                super::PeerObservation {
                    member_id: "node-a".to_string(),
                    sampled: Ok(super::SampledNodeState {
                        state: seed_state.clone(),
                        debug: None,
                    }),
                },
            ),
            (
                "node-b".to_string(),
                super::PeerObservation {
                    member_id: "node-b".to_string(),
                    sampled: Ok(super::SampledNodeState {
                        state: other_state,
                        debug: None,
                    }),
                },
            ),
        ]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, false);

        assert_eq!(view.health, super::ClusterHealth::Degraded);
        assert!(view
            .warnings
            .iter()
            .any(|warning| warning.code == "multi_primary"));
        assert!(view
            .warnings
            .iter()
            .any(|warning| warning.code == "leader_mismatch"));
    }

    #[test]
    fn assemble_cluster_view_marks_degraded_trust_as_degraded() {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FailSafe,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: seed_state.clone(),
                    debug: None,
                }),
            },
        )]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, false);

        assert_eq!(view.health, super::ClusterHealth::Degraded);
        assert!(view
            .warnings
            .iter()
            .any(|warning| warning.code == "degraded_trust"));
    }

    #[test]
    fn assemble_cluster_view_preserves_verbose_mode_without_debug_payload() {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: seed_state.clone(),
                    debug: None,
                }),
            },
        )]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, true);

        assert!(view.verbose);
        assert_eq!(view.nodes[0].pginfo, None);
    }

    #[test]
    fn assemble_cluster_view_preserves_debug_observation_reasons() {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: seed_state.clone(),
                    debug: Some(ClusterNodeDebugObservation {
                        status: DebugObservationStatus::AuthFailed,
                        detail: Some("http 401: missing token".to_string()),
                        payload: None,
                    }),
                }),
            },
        )]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, true);

        assert!(view.verbose);
        assert_eq!(view.nodes[0].pginfo, None);
        assert_eq!(
            view.nodes[0].debug.as_ref().map(|value| &value.status),
            Some(&DebugObservationStatus::AuthFailed)
        );
    }

    #[test]
    fn assemble_cluster_view_includes_debug_payload_summary_when_available() {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: seed_state.clone(),
                    debug: Some(ClusterNodeDebugObservation {
                        status: DebugObservationStatus::Available,
                        detail: None,
                        payload: Some(sample_debug_payload("node-a")),
                    }),
                }),
            },
        )]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, true);

        assert_eq!(
            view.nodes[0].pginfo.as_deref(),
            Some("primary wal_lsn=7 readiness=Ready")
        );
        assert_eq!(view.nodes[0].process.as_deref(), Some("idle"));
        assert_eq!(
            view.nodes[0].debug.as_ref().map(|value| &value.status),
            Some(&DebugObservationStatus::Available)
        );
    }
}
