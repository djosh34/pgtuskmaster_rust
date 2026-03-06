use serde::Serialize;

use crate::cli::{
    args::OutputFormat,
    client::{AcceptedResponse, HaDecisionResponse, HaStateResponse},
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
        CommandOutput::HaState(value) => OutputRef::State(value.as_ref()),
        CommandOutput::Accepted(value) => OutputRef::Accepted(value),
    };

    serde_json::to_string_pretty(&payload)
        .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
}

fn render_text(command_output: &CommandOutput) -> String {
    match command_output {
        CommandOutput::Accepted(value) => format!("accepted={}", value.accepted),
        CommandOutput::HaState(value) => {
            let value = value.as_ref();
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
                format!("ha_decision={}", render_decision_text(&value.ha_decision)),
                format!("snapshot_sequence={}", value.snapshot_sequence),
            ]
            .join("\n")
        }
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
        HaDecisionResponse::AttemptLeadership => "attempt_leadership".to_string(),
        HaDecisionResponse::FollowLeader { leader_member_id } => {
            format!("follow_leader(leader_member_id={leader_member_id})")
        }
        HaDecisionResponse::BecomePrimary { promote } => {
            format!("become_primary(promote={promote})")
        }
        HaDecisionResponse::StepDown {
            reason,
            release_leader_lease,
            clear_switchover,
            fence,
        } => format!(
            "step_down(reason={reason}, release_leader_lease={release_leader_lease}, clear_switchover={clear_switchover}, fence={fence})"
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
    use crate::cli::{
        args::OutputFormat,
        client::{AcceptedResponse, HaDecisionResponse, HaStateResponse},
        output::render_output,
        CommandOutput,
    };

    #[test]
    fn text_output_renders_state_lines() {
        let output = render_output(
            &CommandOutput::HaState(Box::new(HaStateResponse {
                cluster_name: "cluster-a".to_string(),
                scope: "scope-a".to_string(),
                self_member_id: "node-a".to_string(),
                leader: Some("node-a".to_string()),
                switchover_requested_by: None,
                member_count: 3,
                dcs_trust: crate::api::DcsTrustResponse::FullQuorum,
                ha_phase: crate::api::HaPhaseResponse::Primary,
                ha_tick: 9,
                ha_decision: HaDecisionResponse::BecomePrimary { promote: true },
                snapshot_sequence: 77,
            })),
            OutputFormat::Text,
        );
        assert!(output.is_ok(), "text render should succeed");
        let rendered = output.unwrap_or_default();
        assert!(!rendered.is_empty(), "rendered text should not be empty");
        assert!(rendered.contains("cluster_name=cluster-a"));
        assert!(rendered.contains("leader=node-a"));
        assert!(rendered.contains("switchover_requested_by=<none>"));
        assert!(rendered.contains("ha_decision=become_primary(promote=true)"));
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
