use std::{io::Write, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    api::NodeState,
    cli::{
        args::StatusOptions, client::CliApiClient, config::OperatorContext, error::CliError, output,
    },
    dcs::{DcsMemberPostgresView, DcsMemberView, DcsSwitchoverStateView, DcsSwitchoverTargetView, DcsTrust},
    ha::types::{AuthorityProjection, PublicationState},
    pginfo::state::Readiness,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterHealth {
    Healthy,
    Degraded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiStatus {
    Ok,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryOrigin {
    pub member_id: String,
    pub api_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterSwitchoverView {
    pub pending: bool,
    pub target_member_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterNodeView {
    pub member_id: String,
    pub is_self: bool,
    pub api_url: Option<String>,
    pub api_status: ApiStatus,
    pub role: String,
    pub trust: String,
    pub phase: Option<String>,
    pub leader: Option<String>,
    pub decision: Option<String>,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub readiness: String,
    pub process: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterStatusView {
    pub cluster_name: String,
    pub scope: String,
    pub verbose: bool,
    pub queried_via: QueryOrigin,
    pub discovered_member_count: usize,
    pub health: ClusterHealth,
    pub warnings: Vec<ClusterWarning>,
    pub switchover: Option<ClusterSwitchoverView>,
    pub nodes: Vec<ClusterNodeView>,
}

pub(crate) async fn run_status(
    context: &OperatorContext,
    options: StatusOptions,
) -> Result<String, CliError> {
    if options.watch {
        run_watch(context, options).await
    } else if options.json {
        render_state_json(context).await
    } else {
        let view = build_cluster_status_view(context, options).await?;
        output::render_status_view(&view, false)
    }
}

pub(crate) async fn build_cluster_status_view(
    context: &OperatorContext,
    options: StatusOptions,
) -> Result<ClusterStatusView, CliError> {
    let (state, queried_via) = fetch_seed_state(context).await?;
    Ok(assemble_cluster_view(&state, queried_via, options.verbose))
}

pub(crate) async fn fetch_seed_state(
    context: &OperatorContext,
) -> Result<(NodeState, QueryOrigin), CliError> {
    let client = CliApiClient::from_config(context.api_client.clone())?;
    let state = client.get_state().await?;
    let queried_via = QueryOrigin {
        member_id: state.self_member_id.clone(),
        api_url: client.base_url().to_string(),
    };
    Ok((state, queried_via))
}

async fn run_watch(context: &OperatorContext, options: StatusOptions) -> Result<String, CliError> {
    let mut stdout = std::io::stdout();
    let interval = Duration::from_secs(2);

    loop {
        if options.json {
            let rendered = render_state_json(context).await?;
            writeln!(stdout, "{rendered}")
                .map_err(|err| CliError::Output(format!("watch write failed: {err}")))?;
        } else {
            let view = build_cluster_status_view(context, options).await?;
            let rendered = output::render_status_view(&view, false)?;
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

async fn render_state_json(context: &OperatorContext) -> Result<String, CliError> {
    let (state, _queried_via) = fetch_seed_state(context).await?;
    serde_json::to_string_pretty(&state)
        .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
}

fn assemble_cluster_view(
    state: &NodeState,
    queried_via: QueryOrigin,
    verbose: bool,
) -> ClusterStatusView {
    let warnings = collect_warnings(state);
    let health = if warnings.is_empty() {
        ClusterHealth::Healthy
    } else {
        ClusterHealth::Degraded
    };

    let mut nodes = state
        .dcs
        .members
        .values()
        .map(|member| build_node_view(state, member))
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| {
        right
            .is_self
            .cmp(&left.is_self)
            .then_with(|| left.member_id.cmp(&right.member_id))
    });

    ClusterStatusView {
        cluster_name: state.cluster_name.clone(),
        scope: state.scope.clone(),
        verbose,
        queried_via,
        discovered_member_count: nodes.len(),
        health,
        warnings,
        switchover: state
            .dcs
            .switchover
            .clone()
            .into_requested()
            .map(|switchover| ClusterSwitchoverView {
                pending: true,
                target_member_id: match &switchover.target {
                    DcsSwitchoverTargetView::AnyHealthyReplica => None,
                    DcsSwitchoverTargetView::Specific(member_id) => Some(member_id.0.clone()),
                },
            }),
        nodes,
    }
}

fn collect_warnings(state: &NodeState) -> Vec<ClusterWarning> {
    let mut warnings = Vec::new();
    if state.dcs.trust != DcsTrust::FullQuorum {
        warnings.push(ClusterWarning {
            code: "degraded_trust".to_string(),
            message: format!(
                "seed node reports {} DCS trust",
                dcs_trust_label(&state.dcs.trust)
            ),
        });
    }
    if authority_primary_member(state).is_none() {
        warnings.push(ClusterWarning {
            code: "no_primary".to_string(),
            message: "seed node does not currently project an authoritative primary".to_string(),
        });
    }
    if state.dcs.members.is_empty() {
        warnings.push(ClusterWarning {
            code: "no_members".to_string(),
            message: "seed node does not currently expose any DCS member slots".to_string(),
        });
    }
    warnings
}

fn build_node_view(state: &NodeState, member: &DcsMemberView) -> ClusterNodeView {
    let is_self = member.member_id.0 == state.self_member_id;
    ClusterNodeView {
        member_id: member.member_id.0.clone(),
        is_self,
        api_url: member
            .routing
            .api
            .as_ref()
            .map(|endpoint| endpoint.url.clone()),
        api_status: ApiStatus::Ok,
        role: member_role_label(&member.postgres).to_string(),
        trust: dcs_trust_label(&state.dcs.trust).to_string(),
        phase: is_self.then(|| state.ha.role.label().to_string()),
        leader: authority_primary_member(state),
        decision: is_self.then(|| authority_label(&state.ha.publication)),
        postgres_host: member.routing.postgres.host.clone(),
        postgres_port: member.routing.postgres.port,
        readiness: member_readiness_label(&member.postgres).to_string(),
        process: is_self.then(|| format!("{:?}", state.process).to_lowercase()),
    }
}

pub(crate) fn authority_primary_member(state: &NodeState) -> Option<String> {
    match &state.ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            Some(epoch.holder.0.clone())
        }
        PublicationState::Unknown
        | PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => None,
    }
}

fn member_role_label(member: &DcsMemberPostgresView) -> &'static str {
    match member {
        DcsMemberPostgresView::Unknown(_) => "unknown",
        DcsMemberPostgresView::Primary(_) => "primary",
        DcsMemberPostgresView::Replica(_) => "replica",
    }
}

fn member_readiness_label(member: &DcsMemberPostgresView) -> &'static str {
    let readiness = match member {
        DcsMemberPostgresView::Unknown(observation) => &observation.readiness,
        DcsMemberPostgresView::Primary(observation) => &observation.readiness,
        DcsMemberPostgresView::Replica(observation) => &observation.readiness,
    };
    match readiness {
        Readiness::Unknown => "unknown",
        Readiness::Ready => "ready",
        Readiness::NotReady => "not_ready",
    }
}

pub(crate) fn member_is_ready_replica(member: &DcsMemberView) -> bool {
    matches!(
        &member.postgres,
        DcsMemberPostgresView::Replica(observation) if observation.readiness == Readiness::Ready
    )
}

fn authority_label(publication: &PublicationState) -> String {
    match publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            format!("primary({})", epoch.holder.0)
        }
        PublicationState::Projected(AuthorityProjection::NoPrimary(reason)) => {
            format!("no_primary({reason:?})").to_lowercase()
        }
        PublicationState::Unknown => "unknown".to_string(),
    }
}

fn dcs_trust_label(trust: &DcsTrust) -> &'static str {
    match trust {
        DcsTrust::FullQuorum => "full_quorum",
        DcsTrust::Degraded => "degraded",
        DcsTrust::NotTrusted => "not_trusted",
    }
}

trait DcsSwitchoverStateViewExt {
    fn into_requested(self) -> Option<crate::dcs::DcsSwitchoverView>;
}

impl DcsSwitchoverStateViewExt for DcsSwitchoverStateView {
    fn into_requested(self) -> Option<crate::dcs::DcsSwitchoverView> {
        match self {
            DcsSwitchoverStateView::None => None,
            DcsSwitchoverStateView::Requested(view) => Some(view),
        }
    }
}
