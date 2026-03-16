use std::path::Path;

use pgtuskmaster_rust::{
    api::NodeState,
    command::{CommandOutputDto, RenderedConnectionCommandDto},
    dcs::{DcsMode, MemberPostgresView},
    ha::types::{AuthorityProjection, PublicationState},
};
use serde::de::DeserializeOwned;

pub use pgtuskmaster_rust::command::materialize_connection_dsn;

use crate::support::{
    docker::cli::DockerCli,
    error::{HarnessError, Result},
    topology::ClusterMember,
};

const PGTM_BIN: &str = "/usr/local/bin/pgtm";

pub type ClusterStatusView = NodeState;

#[derive(Clone, Debug)]
struct SelectedSeed {
    member: ClusterMember,
    state: ClusterStatusView,
}

#[derive(Clone, Debug)]
pub struct PgtmObserver {
    docker: DockerCli,
    observer_container: String,
}

impl PgtmObserver {
    pub fn new(docker: DockerCli, observer_container: String) -> Self {
        Self {
            docker,
            observer_container,
        }
    }

    pub fn state(&self) -> Result<ClusterStatusView> {
        self.select_seed().map(|seed| seed.state)
    }

    pub fn state_via_member(&self, member: ClusterMember) -> Result<ClusterStatusView> {
        let config = config_path(member);
        let output = self.run(config, &["status", "--json"])?;
        parse_state_output(
            output.as_str(),
            format!("pgtm status via {}", config.display()),
        )
    }

    pub fn primary_tls_json(&self) -> Result<RenderedConnectionCommandDto> {
        self.run_selected_view(["primary", "--json", "--tls"].as_slice(), "pgtm primary --tls")
    }

    pub fn replicas_tls_json(&self) -> Result<RenderedConnectionCommandDto> {
        self.run_selected_view(
            ["replicas", "--json", "--tls"].as_slice(),
            "pgtm replicas --tls",
        )
    }

    pub fn state_and_primary_tls_json(&self) -> Result<(ClusterStatusView, RenderedConnectionCommandDto)> {
        let seed = self.select_seed()?;
        let config = config_path(seed.member);
        let output = self.run(config, ["primary", "--json", "--tls"].as_slice())?;
        let primary =
            parse_connection_output(output.as_str(), format!("pgtm primary --tls via {}", config.display()))?;
        Ok((seed.state, primary))
    }

    pub fn switchover_request_via_member(
        &self,
        member: ClusterMember,
        target: Option<ClusterMember>,
    ) -> Result<String> {
        let config = config_path(member);
        let args = match target {
            Some(target_member) => vec![
                "--json",
                "switchover",
                "request",
                "--switchover-to",
                target_member.service_name(),
            ],
            None => vec!["--json", "switchover", "request"],
        };
        self.run(config, args.as_slice())
    }

    fn run(&self, config: &Path, args: &[&str]) -> Result<String> {
        let mut all_args = vec![
            "-c",
            config.to_str().ok_or_else(|| {
                HarnessError::message(format!(
                    "observer config path is not valid utf-8: {}",
                    config.display()
                ))
            })?,
        ];
        all_args.extend(args.iter().copied());
        self.docker.exec(
            self.observer_container.as_str(),
            Path::new(PGTM_BIN),
            all_args.as_slice(),
        )
    }

    fn select_seed(&self) -> Result<SelectedSeed> {
        let mut best_seed = None;
        let mut best_score = None;
        let mut errors = Vec::new();
        for member in config_paths() {
            let config = config_path(member);
            match self.run(config, &["status", "--json"]).and_then(|output| {
                parse_state_output(
                    output.as_str(),
                    format!("pgtm status via {}", config.display()),
                )
            }) {
                Ok(state) => {
                    let score = status_score(&state);
                    match best_score {
                        Some(previous) if previous >= score => {}
                        _ => {
                            best_score = Some(score);
                            best_seed = Some(SelectedSeed { member, state });
                        }
                    }
                }
                Err(err) => errors.push(format!("{}: {err}", member.service_name())),
            }
        }
        best_seed.ok_or_else(|| aggregate_seed_failure("pgtm status", &errors))
    }

    fn run_selected_view(&self, args: &[&str], operation: &str) -> Result<RenderedConnectionCommandDto> {
        let seed = self.select_seed()?;
        let config = config_path(seed.member);
        let output = self.run(config, args)?;
        parse_connection_output(output.as_str(), format!("{operation} via {}", config.display()))
    }
}

fn config_paths() -> [ClusterMember; 3] {
    ClusterMember::ALL
}

fn config_path(member: ClusterMember) -> &'static Path {
    Path::new(member.observer_config_path())
}

fn parse_json<T>(input: &str, context: impl Into<String>) -> Result<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(input).map_err(|source| HarnessError::Json {
        context: context.into(),
        source,
    })
}

fn parse_state_output(input: &str, context: impl Into<String>) -> Result<NodeState> {
    let context = context.into();
    let output = parse_json::<CommandOutputDto>(input, context.clone())?;
    match output {
        CommandOutputDto::State { output } => Ok(output.state),
        other => Err(HarnessError::message(format!(
            "{context} returned `{}` instead of a state command payload",
            command_label(&other)
        ))),
    }
}

fn parse_connection_output(
    input: &str,
    context: impl Into<String>,
) -> Result<RenderedConnectionCommandDto> {
    let context = context.into();
    let output = parse_json::<CommandOutputDto>(input, context.clone())?;
    match output {
        CommandOutputDto::Primary { output } | CommandOutputDto::Replicas { output } => Ok(output),
        other => Err(HarnessError::message(format!(
            "{context} returned `{}` instead of a connection command payload",
            command_label(&other)
        ))),
    }
}

fn command_label(output: &CommandOutputDto) -> &'static str {
    match output {
        CommandOutputDto::State { .. } => "state",
        CommandOutputDto::Primary { .. } => "primary",
        CommandOutputDto::Replicas { .. } => "replicas",
        CommandOutputDto::Switchover { .. } => "switchover",
        CommandOutputDto::ReloadCertificates { .. } => "reload_certificates",
    }
}

fn aggregate_seed_failure(operation: &str, errors: &[String]) -> HarnessError {
    HarnessError::message(format!(
        "{operation} failed for every observer seed:\n{}",
        errors.join("\n")
    ))
}

fn status_score(status: &ClusterStatusView) -> (usize, usize, usize, usize) {
    let reported_primary_count = status
        .dcs
        .cluster()
        .into_iter()
        .flat_map(|cluster| cluster.members())
        .filter(|(_member_id, member)| {
            matches!(member.postgres(), MemberPostgresView::Primary { .. })
        })
        .count();
    let discovered_member_count = status
        .dcs
        .cluster()
        .map(|cluster| cluster.member_count())
        .unwrap_or_default();
    (
        usize::from(status.dcs.mode() == DcsMode::Coordinated),
        usize::from(matches!(
            &status.ha.publication,
            PublicationState::Projected(AuthorityProjection::Primary(_))
        )),
        usize::from(reported_primary_count == 1),
        discovered_member_count,
    )
}
