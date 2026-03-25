use crate::{
    api::{authoritative_primary_member, AcceptedResponse, NodeState},
    cli::error::CliError,
    pginfo::state::PgConnInfo,
};

pub(crate) fn render_state_output(
    state: &NodeState,
    api_url: &str,
    verbose: bool,
    json: bool,
) -> Result<String, CliError> {
    render_output(state, json, || format_state_output(state, api_url, verbose))
}

pub(crate) fn render_connection_targets(
    targets: &[PgConnInfo],
    json: bool,
) -> Result<String, CliError> {
    render_output(targets, json, || {
        targets
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    })
}

pub(crate) fn render_accepted_response(
    response: &AcceptedResponse,
    json: bool,
) -> Result<String, CliError> {
    render_output(response, json, || format!("accepted={}", response.accepted))
}

fn render_output<T>(
    value: &T,
    json: bool,
    text_renderer: impl FnOnce() -> String,
) -> Result<String, CliError>
where
    T: serde::Serialize + ?Sized,
{
    if json {
        Ok(serde_json::to_string_pretty(value)?)
    } else {
        Ok(text_renderer())
    }
}

fn format_state_output(state: &NodeState, api_url: &str, verbose: bool) -> String {
    let warnings = state_warnings(state);
    let pending_switchover_target = pending_switchover_target(state);
    let health = if warnings.is_empty() {
        "healthy"
    } else {
        "degraded"
    };
    let headers: Vec<&str> = if verbose {
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
    let warning_lines = warnings
        .iter()
        .map(|warning| format!("warning: {warning}\n"))
        .collect::<String>();
    let switchover_line = pending_switchover_target
        .map(|target_member_id| format!("switchover: pending -> {target_member_id}\n"))
        .unwrap_or_default();
    let details_gap = if warnings.is_empty() && pending_switchover_target.is_none() {
        ""
    } else {
        "\n"
    };
    let table_rows = rows
        .into_iter()
        .map(|row| format_table_line(row.as_slice(), widths.as_slice()))
        .collect::<Vec<_>>()
        .join("\n");
    let table = if table_rows.is_empty() {
        format_table_line(headers.as_slice(), widths.as_slice())
    } else {
        format!(
            "{}\n{}",
            format_table_line(headers.as_slice(), widths.as_slice()),
            table_rows
        )
    };

    format!(
        "cluster: {}  health: {health}\nqueried via: {api_url}\n{warning_lines}{switchover_line}{details_gap}{table}",
        state.identity.cluster_name.0
    )
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

fn format_table_line<T: AsRef<str>>(values: &[T], widths: &[usize]) -> String {
    values
        .iter()
        .zip(widths.iter())
        .enumerate()
        .map(|(index, (value, width))| {
            if index == 0 {
                format!("{:<width$}", value.as_ref(), width = *width)
            } else {
                format!("  {:<width$}", value.as_ref(), width = *width)
            }
        })
        .collect::<String>()
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

    use super::{render_accepted_response, render_connection_targets, render_state_output};

    #[test]
    fn connection_command_display_uses_canonical_conninfo_rendering() -> Result<(), String> {
        let rendered = render_connection_targets(
            &[PgConnInfo {
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
            false,
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(
            rendered,
            "host=db.internal port=5432 user=postgres dbname=postgres sslmode=verify-full sslrootcert='/tmp/ca bundle.pem' sslcert='/tmp/client cert.pem' sslkey='/tmp/client key.pem'"
        );
        Ok(())
    }

    #[test]
    fn accepted_response_render_emits_pretty_json_when_requested() -> Result<(), String> {
        let rendered = render_accepted_response(&AcceptedResponse { accepted: true }, true)
            .map_err(|err| err.to_string())?;

        assert!(!rendered.contains("\"command\""));
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
        let rendered = render_state_output(
            &sample_state(
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
            ),
            "http://node-a:8443",
            false,
            false,
        )
        .map_err(|err| err.to_string())?;

        assert!(rendered.contains("ROLE"));
        assert!(rendered.contains("TRUST"));
        assert!(rendered.contains("READINESS"));
        assert!(rendered.contains("primary"));
        assert!(rendered.contains("quorum"));
        assert!(rendered.contains("ready"));
        Ok(())
    }

    #[test]
    fn state_command_display_warns_with_typed_no_quorum_label() -> Result<(), String> {
        let rendered = render_state_output(
            &sample_state(DcsSnapshot::NoQuorum, Readiness::NotReady),
            "http://node-a:8443",
            false,
            false,
        )
        .map_err(|err| err.to_string())?;

        assert!(rendered.contains("warning: seed node reports no_quorum DCS mode"));
        Ok(())
    }
}
