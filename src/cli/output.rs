use crate::cli::{
    client::AcceptedResponse,
    error::CliError,
    status::{ApiStatus, ClusterHealth, ClusterNodeView, ClusterStatusView},
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

    lines.join("\n")
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

#[cfg(test)]
mod tests {
    use crate::cli::{
        client::AcceptedResponse,
        output::{render_accepted_output, render_status_view},
        status::{ApiStatus, ClusterHealth, ClusterNodeView, ClusterStatusView, QueryOrigin},
    };

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
    }

    #[test]
    fn json_output_renders_accepted_payload() {
        let output = render_accepted_output(&AcceptedResponse { accepted: true }, true);
        assert!(output.is_ok(), "json render should succeed");
        let rendered = output.unwrap_or_default();
        assert!(rendered.contains("\"accepted\": true"));
    }
}
