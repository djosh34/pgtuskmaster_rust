use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    api::{AcceptedResponse, NodeState, ReloadCertificatesResponse},
    dcs::{ClusterMemberView, MemberPostgresView, SwitchoverView},
    ha::types::{AuthorityProjection, PublicationState},
    pginfo::state::{PgConnInfo, Readiness},
    state::{MemberId, SwitchoverState},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum CommandOutputDto {
    State {
        output: Box<StateCommandOutputDto>,
    },
    Primary {
        output: StateDerivedConnectionCommandDto,
    },
    Replicas {
        output: StateDerivedConnectionCommandDto,
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

impl StateCommandOutputDto {
    pub(crate) fn from_seed_state(
        state: NodeState,
        queried_via: StateQueryOriginDto,
        verbose: bool,
    ) -> Self {
        let projection = StateProjectionDto::from_seed_state(&state, queried_via, verbose);
        Self { projection, state }
    }
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

impl StateProjectionDto {
    pub(crate) fn from_seed_state(
        state: &NodeState,
        queried_via: StateQueryOriginDto,
        verbose: bool,
    ) -> Self {
        let warnings = collect_warnings(state);
        let health = if warnings.is_empty() {
            StateHealthDto::Healthy
        } else {
            StateHealthDto::Degraded
        };
        Self {
            cluster_name: state.identity.cluster_name.0.clone(),
            scope: state.identity.scope.0.clone(),
            queried_via,
            health,
            verbose,
            discovered_member_count: state.dcs.member_count(),
            warnings,
            switchover: switchover_projection(&state.dcs),
        }
    }
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
pub struct StateDerivedConnectionCommandDto {
    pub projection: StateProjectionDto,
    pub kind: StateDerivedConnectionCommandKind,
    pub targets: Vec<StateDerivedConnectionTargetDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateDerivedConnectionTargetDto {
    pub member_id: String,
    pub conninfo: PgConnInfo,
}

impl fmt::Display for CommandOutputDto {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State { output } => output.fmt(formatter),
            Self::Primary { output } | Self::Replicas { output } => output.fmt(formatter),
            Self::Switchover { output } => write!(formatter, "accepted={}", output.accepted),
            Self::ReloadCertificates { output } => match serde_json::to_string_pretty(output) {
                Ok(json) => formatter.write_str(json.as_str()),
                Err(_) => formatter.write_str("failed to encode reload certificates response"),
            },
        }
    }
}

impl fmt::Display for StateDerivedConnectionCommandDto {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, target) in self.targets.iter().enumerate() {
            if index > 0 {
                writeln!(formatter)?;
            }
            write!(formatter, "{}", target.conninfo)?;
        }
        Ok(())
    }
}

impl fmt::Display for StateCommandOutputDto {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let projection = &self.projection;
        writeln!(
            formatter,
            "cluster: {}  health: {}",
            projection.cluster_name, projection.health
        )?;
        writeln!(formatter, "queried via: {}", projection.queried_via.api_url)?;

        for warning in &projection.warnings {
            writeln!(formatter, "warning: {}", warning.message)?;
        }

        if let Some(switchover) = &projection.switchover {
            writeln!(
                formatter,
                "switchover: pending -> {}",
                switchover.target_member_id.as_deref().unwrap_or("auto")
            )?;
        }

        if !projection.warnings.is_empty() || projection.switchover.is_some() {
            writeln!(formatter)?;
        }

        let headers: &[&str] = if projection.verbose {
            &[
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
            &["NODE", "SELF", "ROLE", "TRUST", "READINESS", "API"]
        };
        let rows = sorted_state_rows(&self.state, projection.verbose);
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

        fmt_table_line(formatter, headers, widths.as_slice())?;
        for row in rows {
            writeln!(formatter)?;
            fmt_table_line(formatter, row.as_slice(), widths.as_slice())?;
        }
        Ok(())
    }
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
            yes_no(is_self).to_string(),
            member_role_label(member.postgres()).to_string(),
            dcs_mode_label(&state.dcs).to_string(),
            authority_primary_member_label(state)
                .unwrap_or("-")
                .to_string(),
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
            yes_no(is_self).to_string(),
            member_role_label(member.postgres()).to_string(),
            dcs_mode_label(&state.dcs).to_string(),
            readiness_label(&member.postgres().readiness()).to_string(),
            "-".to_string(),
        ]
    }
}

fn fmt_table_line<T: AsRef<str>>(
    formatter: &mut fmt::Formatter<'_>,
    values: &[T],
    widths: &[usize],
) -> fmt::Result {
    for (index, (value, width)) in values.iter().zip(widths.iter()).enumerate() {
        if index > 0 {
            formatter.write_str("  ")?;
        }
        write!(formatter, "{:<width$}", value.as_ref(), width = *width)?;
    }
    Ok(())
}

impl fmt::Display for StateHealthDto {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
        })
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

pub(crate) fn authoritative_primary_member(state: &NodeState) -> Option<&MemberId> {
    match &state.ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => Some(&epoch.holder),
        PublicationState::Unknown
        | PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => None,
    }
}

fn authority_primary_member_label(state: &NodeState) -> Option<&str> {
    authoritative_primary_member(state).map(MemberId::as_str)
}

fn collect_warnings(state: &NodeState) -> Vec<StateWarningDto> {
    let degraded_mode_warning = (!state.dcs.is_quorum()).then(|| StateWarningDto {
        code: "degraded_dcs_mode".to_string(),
        message: format!("seed node reports {} DCS mode", dcs_mode_label(&state.dcs)),
    });
    let no_primary_warning =
        authoritative_primary_member(state)
            .is_none()
            .then(|| StateWarningDto {
                code: "no_primary".to_string(),
                message: "seed node does not currently project an authoritative primary"
                    .to_string(),
            });
    let no_members_warning = (state.dcs.member_count() == 0).then(|| StateWarningDto {
        code: "no_members".to_string(),
        message: "seed node does not currently expose any DCS member slots".to_string(),
    });

    [
        degraded_mode_warning,
        no_primary_warning,
        no_members_warning,
    ]
    .into_iter()
    .flatten()
    .collect()
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        pginfo::{
            conninfo::{PgClientTls, PgSslMode},
            state::PgConnInfo,
        },
        state::PgEndpoint,
    };

    use super::{
        StateDerivedConnectionCommandDto, StateDerivedConnectionCommandKind,
        StateDerivedConnectionTargetDto, StateHealthDto, StateProjectionDto, StateQueryOriginDto,
    };

    fn sample_projection() -> StateProjectionDto {
        StateProjectionDto {
            cluster_name: "cluster-a".to_string(),
            scope: "scope-a".to_string(),
            queried_via: StateQueryOriginDto {
                member_id: "node-a".to_string(),
                api_url: "http://node-a:8443".to_string(),
            },
            health: StateHealthDto::Healthy,
            verbose: false,
            discovered_member_count: 1,
            warnings: Vec::new(),
            switchover: None,
        }
    }

    #[test]
    fn connection_command_display_uses_canonical_conninfo_rendering() -> Result<(), String> {
        let output = StateDerivedConnectionCommandDto {
            projection: sample_projection(),
            kind: StateDerivedConnectionCommandKind::Primary,
            targets: vec![StateDerivedConnectionTargetDto {
                member_id: "node-a".to_string(),
                conninfo: PgConnInfo {
                    endpoint: PgEndpoint::tcp("db.internal".to_string(), 5432)?,
                    hostaddr: None,
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: None,
                    options: None,
                    tls: PgClientTls {
                        mode: PgSslMode::VerifyFull,
                        root_cert: Some(PathBuf::from("/tmp/ca bundle.pem")),
                        client_cert: Some(PathBuf::from("/tmp/client cert.pem")),
                        client_key: Some(PathBuf::from("/tmp/client key.pem")),
                    },
                },
            }],
        };

        assert_eq!(
            output.to_string(),
            "host=db.internal port=5432 user=postgres dbname=postgres sslmode=verify-full sslrootcert='/tmp/ca bundle.pem' sslcert='/tmp/client cert.pem' sslkey='/tmp/client key.pem'"
        );
        Ok(())
    }
}
