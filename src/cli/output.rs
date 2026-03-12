use crate::cli::{
    client::AcceptedResponse,
    connect::ConnectionView,
    debug::DebugVerboseView,
    error::CliError,
    status::{
        ApiStatus, ClusterHealth, ClusterNodeView, ClusterStatusView, DebugObservationStatus,
    },
};

pub fn render_accepted_output(value: &AcceptedResponse, json: bool) -> Result<String, CliError> {
    if json {
        serde_json::to_string_pretty(value)
            .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
    } else {
        Ok(format!("accepted={}", value.accepted))
    }
}

pub fn render_status_view(view: &ClusterStatusView, json: bool) -> Result<String, CliError> {
    if json {
        serde_json::to_string_pretty(view)
            .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
    } else {
        Ok(render_status_text(view))
    }
}

pub fn render_connection_view(view: &ConnectionView, json: bool) -> Result<String, CliError> {
    if json {
        serde_json::to_string_pretty(view)
            .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
    } else {
        render_connection_text(view)
    }
}

pub(crate) fn render_debug_verbose_view(
    view: &DebugVerboseView,
    json: bool,
) -> Result<String, CliError> {
    if json {
        serde_json::to_string_pretty(&view.payload)
            .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
    } else {
        Ok(render_debug_verbose_text(view))
    }
}

fn render_status_text(view: &ClusterStatusView) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "cluster: {}  health: {}",
        view.cluster_name,
        health_label(&view.health)
    ));
    lines.push(format!("queried via: {}", view.queried_via.member_id));

    if let Some(switchover) = view.switchover.as_ref() {
        let target = switchover.target_member_id.as_deref().unwrap_or("auto");
        lines.push(format!("switchover: pending -> {target}"));
    }

    for warning in &view.warnings {
        lines.push(format!("warning: {}", warning.message));
    }

    if !view.warnings.is_empty() || view.switchover.is_some() {
        lines.push(String::new());
    }

    let has_verbose = view.verbose;
    let headers = if has_verbose {
        vec![
            "NODE",
            "SELF",
            "ROLE",
            "TRUST",
            "PHASE",
            "LEADER",
            "DECISION",
            "PGINFO",
            "READINESS",
            "PROCESS",
            "DEBUG",
            "API",
        ]
    } else {
        vec!["NODE", "SELF", "ROLE", "TRUST", "PHASE", "API"]
    };

    let rows = view
        .nodes
        .iter()
        .map(|node| render_row(node, has_verbose))
        .collect::<Vec<_>>();
    let mut widths = headers.iter().map(|value| value.len()).collect::<Vec<_>>();
    for row in &rows {
        for (index, value) in row.iter().enumerate() {
            widths[index] = widths[index].max(value.len());
        }
    }

    lines.push(render_table_line(headers.as_slice(), widths.as_slice()));
    for row in rows {
        lines.push(render_table_line(row.as_slice(), widths.as_slice()));
    }

    if has_verbose {
        let debug_details = render_status_debug_details(view);
        if !debug_details.is_empty() {
            lines.push(String::new());
            lines.extend(debug_details);
        }
    }

    lines.join("\n")
}

fn render_debug_verbose_text(view: &DebugVerboseView) -> String {
    let payload = &view.payload;
    let mut lines = vec![
        format!(
            "member: {}  cluster: {}  scope: {}",
            payload.config.member_id, payload.config.cluster_name, payload.config.scope
        ),
        format!("api url: {}", view.api_url),
        format!(
            "sequence: {}  schema: {}",
            payload.meta.sequence, payload.meta.schema_version
        ),
    ];

    if let Some(since) = view.since {
        lines.push(format!("since: {since}"));
    }

    lines.push(String::new());
    lines.push(format!(
        "pginfo: variant={} sql={} readiness={} summary={}",
        payload.pginfo.variant,
        payload.pginfo.sql,
        payload.pginfo.readiness,
        payload.pginfo.summary
    ));
    lines.push(format!(
        "dcs: trust={} leader={} members={} switchover_intent={}",
        payload.dcs.trust,
        payload.dcs.leader_lease_holder.as_deref().unwrap_or("none"),
        payload.dcs.member_slot_count,
        payload.dcs.has_switchover_intent
    ));
    lines.push(format!(
        "ha: role_intent={} authority={} detail={} planned_commands={}",
        payload.ha.role_intent,
        payload.ha.authority_projection,
        payload.ha.authority_detail.as_deref().unwrap_or("none"),
        payload.ha.planned_commands
    ));
    lines.push(format!(
        "process: state={} worker={} running_job={} last_outcome={}",
        payload.process.state,
        payload.process.worker,
        payload.process.running_job_id.as_deref().unwrap_or("none"),
        payload.process.last_outcome.as_deref().unwrap_or("none")
    ));
    lines.push(format!(
        "debug: history_changes={} history_timeline={} last_sequence={}",
        payload.debug.history_changes, payload.debug.history_timeline, payload.debug.last_sequence
    ));
    lines.push(String::new());
    lines.push("recent changes:".to_string());
    lines.extend(render_recent_changes(
        payload
            .changes
            .iter()
            .rev()
            .take(3)
            .map(|entry| format!("  - #{} {} {}", entry.sequence, entry.domain, entry.summary))
            .collect::<Vec<_>>(),
    ));
    lines.push("recent timeline:".to_string());
    lines.extend(render_recent_changes(
        payload
            .timeline
            .iter()
            .rev()
            .take(3)
            .map(|entry| {
                format!(
                    "  - #{} {} {}",
                    entry.sequence, entry.category, entry.message
                )
            })
            .collect::<Vec<_>>(),
    ));

    lines.join("\n")
}

fn render_connection_text(view: &ConnectionView) -> Result<String, CliError> {
    match view.targets.as_slice() {
        [] => Err(CliError::Output(
            "connection output requires at least one target".to_string(),
        )),
        [target] if view.kind == crate::cli::connect::ConnectionCommandKind::Primary => {
            Ok(target.dsn.clone())
        }
        targets => Ok(targets
            .iter()
            .map(|target| target.dsn.as_str())
            .collect::<Vec<_>>()
            .join("\n")),
    }
}

fn render_status_debug_details(view: &ClusterStatusView) -> Vec<String> {
    let mut lines = vec!["debug details:".to_string()];

    for node in &view.nodes {
        let Some(debug) = node.debug.as_ref() else {
            lines.push(format!("  {}: debug not requested", node.member_id));
            continue;
        };

        lines.push(format!(
            "  {}: debug={}",
            node.member_id,
            debug_observation_label(&debug.status)
        ));

        if let Some(detail) = debug.detail.as_deref() {
            lines.push(format!("    detail: {detail}"));
        }

        match debug.payload.as_ref() {
            Some(payload) => {
                lines.push(format!(
                    "    dcs: trust={} leader={}",
                    payload.dcs.trust,
                    payload.dcs.leader_lease_holder.as_deref().unwrap_or("none")
                ));
                lines.push(format!(
                    "    ha: role_intent={} authority={} detail={}",
                    payload.ha.role_intent,
                    payload.ha.authority_projection,
                    payload.ha.authority_detail.as_deref().unwrap_or("none")
                ));
                lines.push(format!(
                    "    pginfo: variant={} sql={} readiness={} summary={}",
                    payload.pginfo.variant,
                    payload.pginfo.sql,
                    payload.pginfo.readiness,
                    payload.pginfo.summary
                ));
                lines.push(format!(
                    "    process: state={} worker={} running_job={} last_outcome={}",
                    payload.process.state,
                    payload.process.worker,
                    payload.process.running_job_id.as_deref().unwrap_or("none"),
                    payload.process.last_outcome.as_deref().unwrap_or("none")
                ));
                let recent_change_lines = payload
                    .changes
                    .iter()
                    .rev()
                    .take(2)
                    .map(|entry| {
                        format!(
                            "    change #{} {} {}",
                            entry.sequence, entry.domain, entry.summary
                        )
                    })
                    .collect::<Vec<_>>();
                lines.extend(render_recent_changes(recent_change_lines));
                let recent_timeline_lines = payload
                    .timeline
                    .iter()
                    .rev()
                    .take(2)
                    .map(|entry| {
                        format!(
                            "    timeline #{} {} {}",
                            entry.sequence, entry.category, entry.message
                        )
                    })
                    .collect::<Vec<_>>();
                lines.extend(render_recent_changes(recent_timeline_lines));
            }
            None => lines.push("    no debug payload".to_string()),
        }
    }

    lines
}

fn render_recent_changes(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() {
        vec!["  - none".to_string()]
    } else {
        lines
    }
}

fn render_row(node: &ClusterNodeView, verbose: bool) -> Vec<String> {
    let mut row = vec![
        node.member_id.clone(),
        if node.is_self {
            "*".to_string()
        } else {
            String::new()
        },
        node.role.clone(),
        node.trust.clone(),
        node.phase.clone(),
    ];
    if verbose {
        row.extend([
            node.leader.clone().unwrap_or_else(|| "?".to_string()),
            node.decision.clone().unwrap_or_else(|| "?".to_string()),
            node.pginfo.clone().unwrap_or_else(|| "?".to_string()),
            node.readiness.clone().unwrap_or_else(|| "?".to_string()),
            node.process.clone().unwrap_or_else(|| "?".to_string()),
            node.debug
                .as_ref()
                .map(|debug| debug_observation_label(&debug.status).to_string())
                .unwrap_or_else(|| "?".to_string()),
        ]);
    }
    row.push(api_status_label(&node.api_status).to_string());
    row
}

fn render_table_line<T: AsRef<str>>(values: &[T], widths: &[usize]) -> String {
    values
        .iter()
        .zip(widths.iter())
        .map(|(value, width)| format!("{:<width$}", value.as_ref(), width = *width))
        .collect::<Vec<_>>()
        .join("  ")
        .trim_end()
        .to_string()
}

fn health_label(value: &ClusterHealth) -> &'static str {
    match value {
        ClusterHealth::Healthy => "healthy",
        ClusterHealth::Degraded => "degraded",
    }
}

fn api_status_label(value: &ApiStatus) -> &'static str {
    match value {
        ApiStatus::Ok => "ok",
        ApiStatus::Down => "down",
        ApiStatus::Missing => "missing",
    }
}

fn debug_observation_label(value: &DebugObservationStatus) -> &'static str {
    match value {
        DebugObservationStatus::Available => "available",
        DebugObservationStatus::Disabled => "disabled",
        DebugObservationStatus::AuthFailed => "auth_failed",
        DebugObservationStatus::NotReady => "not_ready",
        DebugObservationStatus::TransportFailed => "transport_failed",
        DebugObservationStatus::DecodeFailed => "decode_failed",
        DebugObservationStatus::ApiStatusFailed => "api_status_failed",
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::{
        client::AcceptedResponse,
        connect::{ConnectionCommandKind, ConnectionTarget, ConnectionView},
        debug::DebugVerboseView,
        output::{
            render_accepted_output, render_connection_view, render_debug_verbose_view,
            render_status_view,
        },
        status::{
            ApiStatus, ClusterHealth, ClusterNodeDebugObservation, ClusterNodeView,
            ClusterStatusView, DebugObservationStatus, QueryOrigin,
        },
    };
    use crate::debug_api::view::{
        ApiSection, ConfigSection, DcsSection, DebugChangeView, DebugMeta, DebugSection,
        DebugTimelineView, DebugVerbosePayload, HaSection, PgInfoSection, ProcessSection,
    };

    fn sample_debug_payload(member_id: &str) -> DebugVerbosePayload {
        DebugVerbosePayload {
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
                summary: "primary wal_lsn=7 slots=2 readiness=Ready".to_string(),
            },
            dcs: DcsSection {
                version: 1,
                updated_at_ms: 1,
                worker: "Running".to_string(),
                trust: "FullQuorum".to_string(),
                member_slot_count: 2,
                leader_lease_holder: Some("node-a".to_string()),
                has_switchover_intent: false,
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
                role_intent: "leader".to_string(),
                tick: 1,
                authority_projection: "primary:node-a#1".to_string(),
                authority_detail: Some("already converged".to_string()),
                planned_commands: 0,
            },
            api: ApiSection {
                endpoints: vec!["/debug/verbose".to_string()],
            },
            debug: DebugSection {
                history_changes: 2,
                history_timeline: 2,
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
                message: "promoted primary".to_string(),
            }],
        }
    }

    fn sample_debug_observation(
        status: DebugObservationStatus,
        payload: Option<DebugVerbosePayload>,
    ) -> ClusterNodeDebugObservation {
        ClusterNodeDebugObservation {
            status,
            detail: None,
            payload,
        }
    }

    fn sample_status_view(verbose: bool) -> ClusterStatusView {
        ClusterStatusView {
            cluster_name: "cluster-a".to_string(),
            scope: "scope-a".to_string(),
            verbose,
            queried_via: QueryOrigin {
                member_id: "node-a".to_string(),
                api_url: "http://node-a:8080".to_string(),
            },
            sampled_member_count: 2,
            discovered_member_count: 2,
            health: ClusterHealth::Healthy,
            warnings: Vec::new(),
            switchover: None,
            nodes: vec![
                ClusterNodeView {
                    member_id: "node-a".to_string(),
                    is_self: true,
                    sampled: true,
                    api_url: Some("http://node-a:8080".to_string()),
                    api_status: ApiStatus::Ok,
                    role: "primary".to_string(),
                    trust: "full_quorum".to_string(),
                    phase: "primary".to_string(),
                    leader: verbose.then_some("node-a".to_string()),
                    decision: verbose.then_some("no_change".to_string()),
                    pginfo: verbose.then_some("primary wal_lsn=7".to_string()),
                    readiness: verbose.then_some("ready".to_string()),
                    process: verbose.then_some("idle".to_string()),
                    debug: verbose.then(|| {
                        sample_debug_observation(
                            DebugObservationStatus::Available,
                            Some(sample_debug_payload("node-a")),
                        )
                    }),
                    observation_error: None,
                },
                ClusterNodeView {
                    member_id: "node-b".to_string(),
                    is_self: false,
                    sampled: true,
                    api_url: Some("http://node-b:8080".to_string()),
                    api_status: ApiStatus::Ok,
                    role: "replica".to_string(),
                    trust: "full_quorum".to_string(),
                    phase: "replica".to_string(),
                    leader: verbose.then_some("node-a".to_string()),
                    decision: verbose
                        .then_some("follow_leader(leader_member_id=node-a)".to_string()),
                    pginfo: verbose.then_some("replica replay_lsn=7".to_string()),
                    readiness: verbose.then_some("ready".to_string()),
                    process: verbose.then_some("idle".to_string()),
                    debug: verbose
                        .then(|| sample_debug_observation(DebugObservationStatus::Disabled, None)),
                    observation_error: None,
                },
            ],
        }
    }

    #[test]
    fn human_status_output_renders_compact_table() {
        let rendered = render_status_view(&sample_status_view(false), false);
        assert!(rendered.is_ok(), "text render should succeed");
        let value = rendered.unwrap_or_default();
        assert!(value.contains("cluster: cluster-a  health: healthy"));
        assert!(value.contains("NODE"));
        assert!(value.contains("node-a"));
        assert!(value.contains("full_quorum"));
        assert!(value.contains("ok"));
    }

    #[test]
    fn verbose_status_output_extends_table_columns() {
        let rendered = render_status_view(&sample_status_view(true), false);
        assert!(rendered.is_ok(), "verbose text render should succeed");
        let value = rendered.unwrap_or_default();
        assert!(value.contains("LEADER"));
        assert!(value.contains("PGINFO"));
        assert!(value.contains("PROCESS"));
        assert!(value.contains("DEBUG"));
        assert!(value.contains("debug details:"));
        assert!(value.contains("node-b: debug=disabled"));
    }

    #[test]
    fn debug_verbose_text_output_renders_summary_and_history() {
        let view = DebugVerboseView {
            api_url: "http://node-a:8080".to_string(),
            since: Some(40),
            payload: sample_debug_payload("node-a"),
        };

        let rendered = render_debug_verbose_view(&view, false);
        assert!(rendered.is_ok(), "debug verbose text render should succeed");
        let value = rendered.unwrap_or_default();
        assert!(value.contains("member: node-a  cluster: cluster-a  scope: scope-a"));
        assert!(value.contains("since: 40"));
        assert!(value.contains("recent changes:"));
        assert!(value.contains("recent timeline:"));
    }

    #[test]
    fn debug_verbose_json_output_is_raw_payload() {
        let view = DebugVerboseView {
            api_url: "http://node-a:8080".to_string(),
            since: Some(40),
            payload: sample_debug_payload("node-a"),
        };

        let rendered = render_debug_verbose_view(&view, true);
        assert!(rendered.is_ok(), "debug verbose json render should succeed");
        let value = rendered.unwrap_or_default();
        assert!(value.contains("\"schema_version\": \"v1\""));
        assert!(!value.contains("\"api_url\""));
        assert!(!value.contains("\"since\""));
    }

    #[test]
    fn json_output_renders_accepted_payload() {
        let output = render_accepted_output(&AcceptedResponse { accepted: true }, true);
        assert!(output.is_ok(), "json render should succeed");
        let rendered = output.unwrap_or_default();
        assert!(rendered.contains("\"accepted\": true"));
    }

    #[test]
    fn text_primary_output_renders_single_dsn_line() {
        let view = ConnectionView {
            cluster_name: "cluster-a".to_string(),
            scope: "scope-a".to_string(),
            kind: ConnectionCommandKind::Primary,
            tls: false,
            sampled_member_count: 1,
            discovered_member_count: 1,
            warnings: Vec::new(),
            targets: vec![ConnectionTarget {
                member_id: "node-a".to_string(),
                postgres_host: "node-a.db.example.com".to_string(),
                postgres_port: 5432,
                dsn: "host=node-a.db.example.com port=5432 user=postgres dbname=postgres"
                    .to_string(),
            }],
        };

        let rendered = render_connection_view(&view, false);
        assert!(rendered.is_ok(), "text render should succeed");
        assert_eq!(
            rendered.unwrap_or_default(),
            "host=node-a.db.example.com port=5432 user=postgres dbname=postgres"
        );
    }

    #[test]
    fn text_replicas_output_renders_multiple_lines() {
        let view = ConnectionView {
            cluster_name: "cluster-a".to_string(),
            scope: "scope-a".to_string(),
            kind: ConnectionCommandKind::Replicas,
            tls: false,
            sampled_member_count: 2,
            discovered_member_count: 3,
            warnings: Vec::new(),
            targets: vec![
                ConnectionTarget {
                    member_id: "node-b".to_string(),
                    postgres_host: "node-b.db.example.com".to_string(),
                    postgres_port: 5432,
                    dsn: "host=node-b.db.example.com port=5432 user=postgres dbname=postgres"
                        .to_string(),
                },
                ConnectionTarget {
                    member_id: "node-c".to_string(),
                    postgres_host: "node-c.db.example.com".to_string(),
                    postgres_port: 5432,
                    dsn: "host=node-c.db.example.com port=5432 user=postgres dbname=postgres"
                        .to_string(),
                },
            ],
        };

        let rendered = render_connection_view(&view, false);
        assert!(rendered.is_ok(), "text render should succeed");
        assert_eq!(
            rendered.unwrap_or_default(),
            "host=node-b.db.example.com port=5432 user=postgres dbname=postgres\nhost=node-c.db.example.com port=5432 user=postgres dbname=postgres"
        );
    }
}
