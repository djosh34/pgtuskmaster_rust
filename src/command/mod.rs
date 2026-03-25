use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    api::{authoritative_primary_member, AcceptedResponse, NodeState, ReloadCertificatesResponse},
    cli::error::CliError,
    pginfo::state::PgConnInfo,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum CommandOutputDto {
    State {
        api_url: String,
        verbose: bool,
        state: Box<NodeState>,
    },
    Primary {
        targets: Vec<PgConnInfo>,
    },
    Replicas {
        targets: Vec<PgConnInfo>,
    },
    Switchover {
        output: AcceptedResponse,
    },
    ReloadCertificates {
        output: ReloadCertificatesResponse,
    },
}

impl fmt::Display for CommandOutputDto {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State {
                api_url,
                verbose,
                state,
            } => fmt_state_output(formatter, state.as_ref(), api_url, *verbose),
            Self::Primary { targets } | Self::Replicas { targets } => {
                fmt_connection_targets(formatter, targets.as_slice())
            }
            Self::Switchover { output } => write!(formatter, "accepted={}", output.accepted),
            Self::ReloadCertificates { output } => match serde_json::to_string_pretty(output) {
                Ok(json) => formatter.write_str(json.as_str()),
                Err(_) => formatter.write_str("failed to encode reload certificates response"),
            },
        }
    }
}

impl CommandOutputDto {
    pub fn render(&self, json: bool) -> Result<String, CliError> {
        if json {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(self.to_string())
        }
    }
}

fn fmt_connection_targets(
    formatter: &mut fmt::Formatter<'_>,
    targets: &[PgConnInfo],
) -> fmt::Result {
    for (index, target) in targets.iter().enumerate() {
        if index > 0 {
            writeln!(formatter)?;
        }
        write!(formatter, "{target}")?;
    }
    Ok(())
}

fn fmt_state_output(
    formatter: &mut fmt::Formatter<'_>,
    state: &NodeState,
    api_url: &str,
    verbose: bool,
) -> fmt::Result {
    let warnings = state_warnings(state);
    let pending_switchover_target = pending_switchover_target(state);
    writeln!(
        formatter,
        "cluster: {}  health: {}",
        state.identity.cluster_name.0,
        if warnings.is_empty() {
            "healthy"
        } else {
            "degraded"
        }
    )?;
    writeln!(formatter, "queried via: {api_url}")?;

    for warning in &warnings {
        writeln!(formatter, "warning: {warning}")?;
    }

    if let Some(target_member_id) = pending_switchover_target {
        writeln!(formatter, "switchover: pending -> {target_member_id}")?;
    }

    if !warnings.is_empty() || pending_switchover_target.is_some() {
        writeln!(formatter)?;
    }

    let headers: &[&str] = if verbose {
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
    let authoritative_primary = authoritative_primary_member(state)
        .map(|member_id| member_id.as_str())
        .unwrap_or("-");
    let rows = {
        let mut rows = state
            .dcs
            .members()
            .map(|(member_id, member)| {
                let is_self = member_id == &state.identity.member_id;
                let postgres = member.postgres();
                [
                    member_id.0.clone(),
                    if is_self { "yes" } else { "no" }.to_string(),
                    if postgres.is_primary() {
                        "primary"
                    } else if postgres.upstream().is_some() || postgres.is_ready_replica() {
                        "replica"
                    } else {
                        "unknown"
                    }
                    .to_string(),
                    state.dcs.authority().to_string(),
                ]
                .into_iter()
                .chain(verbose.then(|| authoritative_primary.to_string()))
                .chain([postgres.readiness().to_string()])
                .chain(verbose.then(|| {
                    if is_self {
                        format!("{:?}", state.process).to_lowercase()
                    } else {
                        "-".to_string()
                    }
                }))
                .chain(["-".to_string()])
                .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| right[1].cmp(&left[1]).then_with(|| left[0].cmp(&right[0])));
        rows
    };
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

fn state_warnings(state: &NodeState) -> Vec<String> {
    [
        (!state.dcs.is_quorum())
            .then(|| format!("seed node reports {} DCS mode", state.dcs.authority())),
        authoritative_primary_member(state)
            .is_none()
            .then(|| "seed node does not currently project an authoritative primary".to_string()),
        (state.dcs.member_count() == 0)
            .then(|| "seed node does not currently expose any DCS member slots".to_string()),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn pending_switchover_target(state: &NodeState) -> Option<&str> {
    state
        .dcs
        .switchover()
        .and_then(|switchover| switchover.request())
        .map(|request| {
            request
                .target
                .member()
                .map(|member_id| member_id.as_str())
                .unwrap_or("auto")
        })
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

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::{
        api::{AcceptedResponse, NodeState},
        dcs::{DcsMemberState, DcsSnapshot},
        ha::state::HaState,
        pginfo::{
            conninfo::{PgClientTls, PgSslMode},
            state::{PgConfig, PgConnInfo, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        },
        process::state::ProcessState,
        state::{
            ClusterName, MemberId, NodeIdentity, PgRoute, ScopeName, SwitchoverState, WalLsn,
            WorkerStatus,
        },
    };

    use super::CommandOutputDto;

    #[test]
    fn connection_command_display_uses_canonical_conninfo_rendering() -> Result<(), String> {
        let rendered = CommandOutputDto::Primary {
            targets: vec![PgConnInfo {
                route: PgRoute::tcp("db.internal".to_string(), 5432)?,
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
            }],
        };

        assert_eq!(
            rendered.to_string(),
            "host=db.internal port=5432 user=postgres dbname=postgres sslmode=verify-full sslrootcert='/tmp/ca bundle.pem' sslcert='/tmp/client cert.pem' sslkey='/tmp/client key.pem'"
        );
        Ok(())
    }

    #[test]
    fn command_output_render_uses_display_when_json_disabled() -> Result<(), String> {
        let output = CommandOutputDto::Primary {
            targets: vec![PgConnInfo {
                route: PgRoute::tcp("db.internal".to_string(), 5432)?,
                user: "postgres".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: None,
                options: None,
                tls: PgClientTls {
                    mode: PgSslMode::Disable,
                    root_cert: None,
                    client_cert: None,
                    client_key: None,
                },
            }],
        };

        assert_eq!(
            output.render(false).map_err(|err| err.to_string())?,
            output.to_string()
        );
        Ok(())
    }

    #[test]
    fn command_output_render_emits_pretty_json_when_requested() -> Result<(), String> {
        let output = CommandOutputDto::Switchover {
            output: AcceptedResponse { accepted: true },
        };
        let rendered = output.render(true).map_err(|err| err.to_string())?;

        assert!(rendered.contains("\"command\": \"switchover\""));
        assert!(rendered.contains("\"accepted\": true"));
        assert!(rendered.contains('\n'));
        Ok(())
    }

    fn sample_pg_info(readiness: Readiness) -> PgInfoState {
        PgInfoState::Primary {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness,
                timeline: None,
                system_identifier: None,
                pg_config: PgConfig {
                    port: Some(5432),
                    hot_standby: Some(false),
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: None,
            },
            wal_lsn: WalLsn(42),
            slots: Vec::new(),
        }
    }

    fn sample_state(dcs: DcsSnapshot, readiness: Readiness) -> NodeState {
        let member_id = MemberId("node-a".to_string());
        NodeState {
            identity: NodeIdentity {
                cluster_name: ClusterName("cluster-a".to_string()),
                scope: ScopeName("scope-a".to_string()),
                member_id: member_id.clone(),
            },
            pg: sample_pg_info(readiness.clone()),
            process: ProcessState::Idle {
                worker: WorkerStatus::Running,
                last_outcome: None,
            },
            dcs,
            ha: HaState::initial(WorkerStatus::Running),
        }
    }

    #[test]
    fn state_command_display_renders_typed_trust_and_readiness_labels() -> Result<(), String> {
        let member_id = MemberId("node-a".to_string());
        let rendered = CommandOutputDto::State {
            api_url: "http://node-a:8443".to_string(),
            verbose: false,
            state: Box::new(sample_state(
                DcsSnapshot::quorum(
                    None,
                    SwitchoverState::None,
                    BTreeMap::from([(
                        member_id.clone(),
                        DcsMemberState {
                            cluster_postgres: PgRoute::tcp("db.internal".to_string(), 5432)?,
                            operator_postgres: None,
                            operator_api: None,
                            postgres: sample_pg_info(Readiness::Ready),
                        },
                    )]),
                ),
                Readiness::Ready,
            )),
        };
        let rendered = rendered.to_string();
        assert!(rendered.contains("ROLE"));
        assert!(rendered.contains("TRUST"));
        assert!(rendered.contains("READINESS"));
        assert!(rendered.contains("primary"));
        assert!(rendered.contains("quorum"));
        assert!(rendered.contains("ready"));
        Ok(())
    }

    #[test]
    fn state_command_display_warns_with_typed_no_quorum_label() {
        let rendered = CommandOutputDto::State {
            api_url: "http://node-a:8443".to_string(),
            verbose: false,
            state: Box::new(sample_state(DcsSnapshot::NoQuorum, Readiness::NotReady)),
        }
        .to_string();

        assert!(rendered.contains("warning: seed node reports no_quorum DCS mode"));
    }
}
