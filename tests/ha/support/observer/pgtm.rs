use std::path::PathBuf;

use pgtuskmaster_rust::api::NodeState;
use pgtuskmaster_test_support::ha_runner::{
    RunnerCommand, RunnerResponsePayload, RunnerSeedSelection,
};

pub use pgtuskmaster_rust::cli::connect::ConnectionTarget;

use crate::support::{
    error::{HarnessError, Result},
    runner::run_contract_command,
    runner::{RunnerSeed, RunnerSessionHandle},
    topology::ClusterMember,
};

pub type ClusterStatusView = NodeState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerApiSeed {
    pub member: ClusterMember,
    pub config_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerApiContract {
    pub session: RunnerSessionHandle,
    pub seeds: Vec<RunnerApiSeed>,
}

impl RunnerApiContract {
    pub fn from_session(session: RunnerSessionHandle, seed_set: &[RunnerSeed]) -> Self {
        Self {
            session,
            seeds: seed_set
                .iter()
                .map(|seed| RunnerApiSeed {
                    member: seed.member,
                    config_path: seed.config_path.clone(),
                })
                .collect(),
        }
    }

    pub fn state(&self) -> Result<ClusterStatusView> {
        self.decode_state(RunnerCommand::ClusterStatus {
            seed: RunnerSeedSelection::Automatic,
        })
    }

    pub fn state_via_member(&self, member: ClusterMember) -> Result<ClusterStatusView> {
        self.decode_state(RunnerCommand::ClusterStatus {
            seed: RunnerSeedSelection::ViaMember {
                member_id: member.service_name().to_string(),
            },
        })
    }

    pub fn primary_tls_json(&self) -> Result<pgtuskmaster_rust::cli::connect::ConnectionView> {
        self.decode_connection_view(RunnerCommand::PrimaryTls)
    }

    pub fn replicas_tls_json(&self) -> Result<pgtuskmaster_rust::cli::connect::ConnectionView> {
        self.decode_connection_view(RunnerCommand::ReplicasTls)
    }

    pub fn switchover_request_via_member(
        &self,
        member: ClusterMember,
        target: Option<ClusterMember>,
    ) -> Result<String> {
        match run_contract_command(
            &self.session,
            RunnerCommand::SwitchoverRequest {
                via_member_id: member.service_name().to_string(),
                target_member_id: target.map(|value| value.service_name().to_string()),
            },
        )? {
            RunnerResponsePayload::Accepted { accepted } => serde_json::to_string(&accepted)
                .map_err(|source| HarnessError::Json {
                    context: "serializing runner switchover response".to_string(),
                    source,
                }),
            other => Err(HarnessError::message(format!(
                "runner returned `{}` for switchover request instead of accepted response",
                response_kind_label(&other)
            ))),
        }
    }

    fn decode_state(&self, command: RunnerCommand) -> Result<ClusterStatusView> {
        match run_contract_command(&self.session, command)? {
            RunnerResponsePayload::State { state } => Ok(*state),
            other => Err(HarnessError::message(format!(
                "runner returned `{}` instead of cluster state",
                response_kind_label(&other)
            ))),
        }
    }

    fn decode_connection_view(
        &self,
        command: RunnerCommand,
    ) -> Result<pgtuskmaster_rust::cli::connect::ConnectionView> {
        match run_contract_command(&self.session, command)? {
            RunnerResponsePayload::ConnectionView { view } => Ok(view),
            other => Err(HarnessError::message(format!(
                "runner returned `{}` instead of connection view",
                response_kind_label(&other)
            ))),
        }
    }
}

fn response_kind_label(payload: &RunnerResponsePayload) -> &'static str {
    match payload {
        RunnerResponsePayload::Pong => "pong",
        RunnerResponsePayload::State { .. } => "state",
        RunnerResponsePayload::ConnectionView { .. } => "connection_view",
        RunnerResponsePayload::Accepted { .. } => "accepted",
        RunnerResponsePayload::SqlRows { .. } => "sql_rows",
        RunnerResponsePayload::Text { .. } => "text",
        RunnerResponsePayload::Error { .. } => "error",
    }
}
