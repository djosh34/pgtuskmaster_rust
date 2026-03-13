use crate::cli::{
    client::AcceptedResponse,
    connect::ConnectionView,
    error::CliError,
    status::{ClusterHealth, ClusterStatusView},
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
        Ok(render_connection_text(view))
    }
}

fn render_status_text(view: &ClusterStatusView) -> String {
    let mut lines = vec![
        format!(
            "cluster: {}  health: {}",
            view.cluster_name,
            health_label(&view.health)
        ),
        format!("queried via: {}", view.queried_via.api_url),
    ];

    for warning in &view.warnings {
        lines.push(format!("warning: {}", warning.message));
    }

    if let Some(switchover) = view.switchover.as_ref() {
        let target = switchover.target_member_id.as_deref().unwrap_or("auto");
        lines.push(format!("switchover: pending -> {target}"));
    }

    if !view.warnings.is_empty() || view.switchover.is_some() {
        lines.push(String::new());
    }

    let headers = if view.verbose {
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

    let rows = view
        .nodes
        .iter()
        .map(|node| {
            if view.verbose {
                vec![
                    node.member_id.clone(),
                    yes_no(node.is_self),
                    node.role.clone(),
                    node.trust.clone(),
                    node.leader.clone().unwrap_or_else(|| "-".to_string()),
                    node.readiness.clone(),
                    node.process.clone().unwrap_or_else(|| "-".to_string()),
                    node.api_url.clone().unwrap_or_else(|| "-".to_string()),
                ]
            } else {
                vec![
                    node.member_id.clone(),
                    yes_no(node.is_self),
                    node.role.clone(),
                    node.trust.clone(),
                    node.readiness.clone(),
                    node.api_url.clone().unwrap_or_else(|| "-".to_string()),
                ]
            }
        })
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

fn render_connection_text(view: &ConnectionView) -> String {
    let mut lines = vec![
        format!("cluster: {}", view.cluster_name),
        format!("kind: {:?}", view.kind).to_lowercase(),
    ];

    for warning in &view.warnings {
        lines.push(format!("warning: {}", warning.message));
    }

    if !view.warnings.is_empty() {
        lines.push(String::new());
    }

    for target in &view.targets {
        lines.push(format!("{} {}", target.member_id, target.dsn));
    }

    lines.join("\n")
}

fn render_table_line(values: &[impl AsRef<str>], widths: &[usize]) -> String {
    values
        .iter()
        .zip(widths.iter())
        .map(|(value, width)| format!("{:<width$}", value.as_ref(), width = *width))
        .collect::<Vec<_>>()
        .join("  ")
}

fn health_label(value: &ClusterHealth) -> &'static str {
    match value {
        ClusterHealth::Healthy => "healthy",
        ClusterHealth::Degraded => "degraded",
    }
}

fn yes_no(value: bool) -> String {
    if value {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}
