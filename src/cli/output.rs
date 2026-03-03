use serde::Serialize;

use crate::cli::{
    args::OutputFormat,
    client::{AcceptedResponse, HaStateResponse},
    error::CliError,
    CommandOutput,
};

pub fn render_output(
    command_output: &CommandOutput,
    format: OutputFormat,
) -> Result<String, CliError> {
    match format {
        OutputFormat::Json => render_json(command_output),
        OutputFormat::Text => Ok(render_text(command_output)),
    }
}

fn render_json(command_output: &CommandOutput) -> Result<String, CliError> {
    #[derive(Serialize)]
    #[serde(untagged)]
    enum OutputRef<'a> {
        State(&'a HaStateResponse),
        Accepted(&'a AcceptedResponse),
    }

    let payload = match command_output {
        CommandOutput::HaState(value) => OutputRef::State(value),
        CommandOutput::Accepted(value) => OutputRef::Accepted(value),
    };

    serde_json::to_string_pretty(&payload)
        .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
}

fn render_text(command_output: &CommandOutput) -> String {
    match command_output {
        CommandOutput::Accepted(value) => format!("accepted={}", value.accepted),
        CommandOutput::HaState(value) => {
            let leader = value.leader.as_deref().unwrap_or("<none>");
            let switchover = value.switchover_requested_by.as_deref().unwrap_or("<none>");
            [
                format!("cluster_name={}", value.cluster_name),
                format!("scope={}", value.scope),
                format!("self_member_id={}", value.self_member_id),
                format!("leader={leader}"),
                format!("switchover_requested_by={switchover}"),
                format!("member_count={}", value.member_count),
                format!("dcs_trust={}", value.dcs_trust),
                format!("ha_phase={}", value.ha_phase),
                format!("ha_tick={}", value.ha_tick),
                format!("pending_actions={}", value.pending_actions),
                format!("snapshot_sequence={}", value.snapshot_sequence),
            ]
            .join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::{
        args::OutputFormat,
        client::{AcceptedResponse, HaStateResponse},
        output::render_output,
        CommandOutput,
    };

    #[test]
    fn text_output_renders_state_lines() {
        let output = render_output(
            &CommandOutput::HaState(HaStateResponse {
                cluster_name: "cluster-a".to_string(),
                scope: "scope-a".to_string(),
                self_member_id: "node-a".to_string(),
                leader: Some("node-a".to_string()),
                switchover_requested_by: None,
                member_count: 3,
                dcs_trust: "FullQuorum".to_string(),
                ha_phase: "Primary".to_string(),
                ha_tick: 9,
                pending_actions: 1,
                snapshot_sequence: 77,
            }),
            OutputFormat::Text,
        );
        assert!(output.is_ok(), "text render should succeed");
        let rendered = output.unwrap_or_default();
        assert!(!rendered.is_empty(), "rendered text should not be empty");
        assert!(rendered.contains("cluster_name=cluster-a"));
        assert!(rendered.contains("leader=node-a"));
        assert!(rendered.contains("switchover_requested_by=<none>"));
    }

    #[test]
    fn json_output_renders_accepted_payload() {
        let output = render_output(
            &CommandOutput::Accepted(AcceptedResponse { accepted: true }),
            OutputFormat::Json,
        );
        assert!(output.is_ok(), "json render should succeed");
        let rendered = output.unwrap_or_default();
        assert!(!rendered.is_empty(), "rendered json should not be empty");
        assert!(rendered.contains("\"accepted\": true"));
    }
}
