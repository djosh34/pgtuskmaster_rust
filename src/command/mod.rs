use std::{fmt, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::pginfo::conninfo::render_conninfo_value;

use crate::{
    api::{AcceptedResponse, NodeState, ReloadCertificatesResponse},
    dcs::{ClusterMemberView, MemberPostgresView, SwitchoverView},
    ha::types::{AuthorityProjection, PublicationState},
    pginfo::state::Readiness,
    state::{MemberId, SwitchoverState},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum CommandOutputDto {
    State {
        output: Box<StateCommandOutputDto>,
    },
    Primary {
        output: RenderedConnectionCommandDto,
    },
    Replicas {
        output: RenderedConnectionCommandDto,
    },
    Switchover {
        output: AcceptedResponse,
    },
    ReloadCertificates {
        output: ReloadCertificatesResponse,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateCommandOutputDto {
    pub projection: StateProjectionDto,
    pub state: NodeState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateProjectionDto {
    pub cluster_name: String,
    pub scope: String,
    pub queried_via: StateQueryOriginDto,
    pub health: StateHealthDto,
    pub verbose: bool,
    pub discovered_member_count: usize,
    pub warnings: Vec<StateWarningDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switchover: Option<StateSwitchoverDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateWarningDto {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateQueryOriginDto {
    pub member_id: String,
    pub api_url: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateHealthDto {
    Healthy,
    Degraded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateSwitchoverDto {
    pub pending: bool,
    pub target_member_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateDerivedConnectionCommandKind {
    Primary,
    Replicas,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderedConnectionCommandDto {
    pub state: StateDerivedConnectionCommandDto,
    pub local_connection: LocalConnectionMaterialization,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateDerivedConnectionCommandDto {
    pub projection: StateProjectionDto,
    pub kind: StateDerivedConnectionCommandKind,
    pub targets: Vec<StateDerivedConnectionTargetDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateDerivedConnectionTargetDto {
    pub member_id: String,
    pub postgres_host: String,
    pub postgres_port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LocalConnectionMaterialization {
    Plaintext,
    Tls { paths: PathBackedClientTlsDto },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathBackedClientTlsDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_cert_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key_path: Option<PathBuf>,
}

impl fmt::Display for CommandOutputDto {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = match self {
            Self::State { output } => render_state_command_text(output),
            Self::Primary { output } | Self::Replicas { output } => {
                render_connection_command_text(output)
            }
            Self::Switchover { output } => format!("accepted={}", output.accepted),
            Self::ReloadCertificates { output } => match serde_json::to_string_pretty(output) {
                Ok(json) => json,
                Err(_) => "failed to encode reload certificates response".to_string(),
            },
        };
        formatter.write_str(rendered.as_str())
    }
}

pub fn materialize_connection_dsn(
    target: &StateDerivedConnectionTargetDto,
    local: &LocalConnectionMaterialization,
) -> String {
    let base_fields = [
        ("host", target.postgres_host.clone()),
        ("port", target.postgres_port.to_string()),
        ("user", "postgres".to_string()),
        ("dbname", "postgres".to_string()),
    ];
    let tls_fields = match local {
        LocalConnectionMaterialization::Plaintext => Vec::new(),
        LocalConnectionMaterialization::Tls { paths } => {
            let optional_path_fields = [
                ("sslrootcert", paths.ca_cert_path.as_ref()),
                ("sslcert", paths.client_cert_path.as_ref()),
                ("sslkey", paths.client_key_path.as_ref()),
            ]
            .into_iter()
            .filter_map(|(key, path)| path.map(|value| (key, value.to_string_lossy().into_owned())))
            .collect::<Vec<_>>();
            [("sslmode", "verify-full".to_string())]
                .into_iter()
                .chain(optional_path_fields)
                .collect::<Vec<_>>()
        }
    };

    base_fields
        .into_iter()
        .chain(tls_fields)
        .map(|(key, value)| format!("{key}={}", render_conninfo_value(value.as_str())))
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_connection_command_text(view: &RenderedConnectionCommandDto) -> String {
    view.state
        .targets
        .iter()
        .map(|target| materialize_connection_dsn(target, &view.local_connection))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_state_command_text(output: &StateCommandOutputDto) -> String {
    let projection = &output.projection;
    let header_lines = [
        format!(
            "cluster: {}  health: {}",
            projection.cluster_name,
            health_label(projection.health)
        ),
        format!("queried via: {}", projection.queried_via.api_url),
    ];
    let warning_lines = projection
        .warnings
        .iter()
        .map(|warning| format!("warning: {}", warning.message));
    let switchover_lines = projection
        .switchover
        .as_ref()
        .map(|switchover| {
            format!(
                "switchover: pending -> {}",
                switchover.target_member_id.as_deref().unwrap_or("auto")
            )
        })
        .into_iter();
    let spacer_line = (!projection.warnings.is_empty() || projection.switchover.is_some())
        .then(String::new)
        .into_iter();
    let headers = if projection.verbose {
        vec![
            "NODE",
            "SELF",
            "ROLE",
            "TRUST",
            "LEADER",
            "READINESS",
            "PROCESS",
            "API",
        ]
    } else {
        vec!["NODE", "SELF", "ROLE", "TRUST", "READINESS", "API"]
    };
    let rows = sorted_state_rows(&output.state, projection.verbose);
    let widths = rows.iter().fold(
        headers.iter().map(|value| value.len()).collect::<Vec<_>>(),
        |widths, row| {
            widths
                .into_iter()
                .zip(row.iter())
                .map(|(current, value)| current.max(value.len()))
                .collect::<Vec<_>>()
        },
    );
    let table_lines = std::iter::once(render_table_line(headers.as_slice(), widths.as_slice()))
        .chain(
            rows.into_iter()
                .map(|row| render_table_line(row.as_slice(), widths.as_slice())),
        );

    header_lines
        .into_iter()
        .chain(warning_lines)
        .chain(switchover_lines)
        .chain(spacer_line)
        .chain(table_lines)
        .collect::<Vec<_>>()
        .join("\n")
}

fn sorted_state_rows(state: &NodeState, verbose: bool) -> Vec<Vec<String>> {
    let mut rows = state
        .dcs
        .members()
        .map(|(member_id, member)| state_row(state, member_id, member, verbose))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| right[1].cmp(&left[1]).then_with(|| left[0].cmp(&right[0])));
    rows
}

fn state_row(
    state: &NodeState,
    member_id: &MemberId,
    member: &ClusterMemberView,
    verbose: bool,
) -> Vec<String> {
    let is_self = member_id == &state.identity.member_id;
    if verbose {
        vec![
            member_id.0.clone(),
            yes_no(is_self),
            member_role_label(member.postgres()).to_string(),
            dcs_mode_label(&state.dcs).to_string(),
            authority_primary_member(state).unwrap_or_else(|| "-".to_string()),
            readiness_label(&member.postgres().readiness()).to_string(),
            if is_self {
                format!("{:?}", state.process).to_lowercase()
            } else {
                "-".to_string()
            },
            "-".to_string(),
        ]
    } else {
        vec![
            member_id.0.clone(),
            yes_no(is_self),
            member_role_label(member.postgres()).to_string(),
            dcs_mode_label(&state.dcs).to_string(),
            readiness_label(&member.postgres().readiness()).to_string(),
            "-".to_string(),
        ]
    }
}

fn render_table_line(values: &[impl AsRef<str>], widths: &[usize]) -> String {
    values
        .iter()
        .zip(widths.iter())
        .map(|(value, width)| format!("{:<width$}", value.as_ref(), width = *width))
        .collect::<Vec<_>>()
        .join("  ")
}

fn health_label(value: StateHealthDto) -> &'static str {
    match value {
        StateHealthDto::Healthy => "healthy",
        StateHealthDto::Degraded => "degraded",
    }
}

fn yes_no(value: bool) -> String {
    if value {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

fn authority_primary_member(state: &NodeState) -> Option<String> {
    match &state.ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            Some(epoch.holder.0.clone())
        }
        PublicationState::Unknown
        | PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => None,
    }
}

fn member_role_label(member: &MemberPostgresView) -> &'static str {
    if member.is_primary() {
        "primary"
    } else if member.upstream().is_some() || member.is_ready_replica() {
        "replica"
    } else {
        "unknown"
    }
}

fn readiness_label(readiness: &Readiness) -> &'static str {
    match readiness {
        Readiness::Unknown => "unknown",
        Readiness::Ready => "ready",
        Readiness::NotReady => "not_ready",
    }
}

fn dcs_mode_label(snapshot: &crate::dcs::DcsView) -> &'static str {
    snapshot.mode_label()
}

pub fn switchover_projection(snapshot: &crate::dcs::DcsView) -> Option<StateSwitchoverDto> {
    match snapshot.switchover() {
        Some(SwitchoverView::AnyHealthyReplica) => Some(StateSwitchoverDto {
            pending: true,
            target_member_id: None,
        }),
        Some(SwitchoverView::Specific(member_id)) => Some(StateSwitchoverDto {
            pending: true,
            target_member_id: Some(member_id.0.clone()),
        }),
        Some(SwitchoverState::None) | None => None,
    }
}
