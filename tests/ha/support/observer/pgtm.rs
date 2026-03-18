use std::{
    collections::BTreeMap,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use pgtuskmaster_rust::{
    api::{AcceptedResponse, NodeState},
    command::CommandOutputDto,
    pginfo::conninfo::render_conninfo_value,
};

use crate::support::{
    config::harness_settings,
    docker::cli::DockerCli,
    error::{HarnessError, Result},
    process,
    topology::ClusterMember,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresRoutingTarget {
    pub member: ClusterMember,
    pub dsn: String,
}

pub type ClusterStateObservation = BTreeMap<ClusterMember, std::result::Result<NodeState, String>>;

#[derive(Clone, Debug)]
pub struct PgtmObserver {
    docker: DockerCli,
    compose_file: PathBuf,
    compose_project: String,
    materialized_dir: PathBuf,
}

impl PgtmObserver {
    pub fn new(
        docker: DockerCli,
        compose_file: PathBuf,
        compose_project: String,
        materialized_dir: PathBuf,
    ) -> Self {
        Self {
            docker,
            compose_file,
            compose_project,
            materialized_dir,
        }
    }

    pub fn observe_states(&self) -> Result<ClusterStateObservation> {
        ClusterMember::ALL
            .into_iter()
            .map(|member| Ok((member, self.observe_state_via_member(member))))
            .collect()
    }

    pub fn state_via_member(&self, member: ClusterMember) -> Result<NodeState> {
        self.observe_state_via_member(member).map_err(|message| {
            HarnessError::message(format!("pgtm status via `{member}` failed: {message}"))
        })
    }

    pub fn postgres_routing_target(&self, member: ClusterMember) -> Result<PostgresRoutingTarget> {
        let published_port = self.member_published_port(member, "5432/tcp")?;
        let ca_cert_path = self.materialized_dir.join("configs/tls/ca.crt");
        let observer_cert_path = self.materialized_dir.join("configs/tls/observer.crt");
        let observer_key_path = self.materialized_dir.join("configs/tls/observer.key");
        Ok(PostgresRoutingTarget {
            member,
            dsn: host_postgres_dsn(
                member,
                published_port,
                ca_cert_path.as_path(),
                observer_cert_path.as_path(),
                observer_key_path.as_path(),
            ),
        })
    }

    pub fn switchover_request_via_member(
        &self,
        member: ClusterMember,
        target: Option<ClusterMember>,
    ) -> Result<String> {
        let runtime_config = self.materialize_host_observer_config(member)?;
        let request_args = target
            .into_iter()
            .flat_map(|target_member| {
                [
                    "--switchover-to".to_string(),
                    target_member.service_name().to_string(),
                ]
            })
            .collect::<Vec<_>>();
        let output = self.run_command_via_member(
            member,
            runtime_config.as_path(),
            ["switchover".to_string(), "request".to_string()]
                .into_iter()
                .chain(request_args)
                .collect::<Vec<_>>(),
            "pgtm switchover request",
            extract_switchover_output,
        )?;
        match output {
            Ok(accepted) => serde_json::to_string(&accepted).map_err(|source| {
                HarnessError::message(format!("serializing switchover response failed: {source}"))
            }),
            Err(message) => Err(HarnessError::message(format!(
                "pgtm switchover request via `{member}` failed: {message}"
            ))),
        }
    }

    fn observe_state_via_member(
        &self,
        member: ClusterMember,
    ) -> std::result::Result<NodeState, String> {
        let runtime_config = self
            .materialize_host_observer_config(member)
            .map_err(|err| err.to_string())?;
        self.run_command_via_member(
            member,
            runtime_config.as_path(),
            vec!["status".to_string()],
            "pgtm status",
            extract_state_command_output,
        )
        .map_err(|err| err.to_string())?
    }

    fn run_command_via_member<T>(
        &self,
        member: ClusterMember,
        runtime_config: &Path,
        command_args: Vec<String>,
        context_label: &str,
        decode_output: fn(CommandOutputDto) -> Result<T>,
    ) -> Result<std::result::Result<T, String>> {
        let env_candidate = std::env::var_os("CARGO_BIN_EXE_pgtm")
            .map(PathBuf::from)
            .filter(|path| path.exists());
        let executable = match env_candidate {
            Some(path) => path,
            None => harness_settings()?.pgtm_executable().to_path_buf(),
        };
        process::ensure_absolute_executable(executable.as_path())?;
        let args = [
            "--config".to_string(),
            runtime_config.display().to_string(),
            "--json".to_string(),
        ]
        .into_iter()
        .chain(command_args)
        .collect::<Vec<_>>();
        let context = format!("{context_label} via `{member}`");
        let output = process::run(
            executable.as_path(),
            context.clone(),
            args.as_slice(),
            [("PATH", "")],
        );
        match output {
            Ok(stdout) => {
                let dto = serde_json::from_str::<CommandOutputDto>(stdout.as_str()).map_err(
                    |source| HarnessError::Json {
                        context: context.clone(),
                        source,
                    },
                )?;
                decode_output(dto).map(Ok)
            }
            Err(HarnessError::CommandFailed {
                executable,
                context,
                status,
                stdout,
                stderr,
            }) => Ok(Err(format!(
                "command `{}` failed while {context}: status={status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
                executable.display()
            ))),
            Err(err) => Err(err),
        }
    }

    fn member_published_port(&self, member: ClusterMember, port: &str) -> Result<u16> {
        let container_id = self.docker.compose_container_id(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            member.service_name(),
        )?;
        self.docker.published_host_port(container_id.as_str(), port)
    }

    fn materialize_host_observer_config(&self, member: ClusterMember) -> Result<PathBuf> {
        let published_api_port = self.member_published_port(member, "8443/tcp")?;
        let config_path = self
            .materialized_dir
            .join("configs/observer")
            .join(format!("{}-pgtm.toml", member.service_name()));
        let ca_cert_path = self.materialized_dir.join("configs/tls/ca.crt");
        let read_token_path = self.materialized_dir.join("secrets/api-read-token");
        let admin_token_path = self.materialized_dir.join("secrets/api-admin-token");
        let observer_cert_path = self.materialized_dir.join("configs/tls/observer.crt");
        let observer_key_path = self.materialized_dir.join("configs/tls/observer.key");
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|source| HarnessError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let rendered = render_host_observer_config(
            member,
            SocketAddr::from(([127, 0, 0, 1], published_api_port)),
            ca_cert_path.as_path(),
            read_token_path.as_path(),
            admin_token_path.as_path(),
            observer_cert_path.as_path(),
            observer_key_path.as_path(),
        );
        fs::write(config_path.as_path(), rendered).map_err(|source| HarnessError::Io {
            path: config_path.clone(),
            source,
        })?;
        Ok(config_path)
    }
}

fn extract_state_command_output(output: CommandOutputDto) -> Result<NodeState> {
    match output {
        CommandOutputDto::State { output } => Ok(output.state),
        other => Err(HarnessError::message(format!(
            "expected `pgtm status --json` output, observed command payload `{}`",
            command_label(&other)
        ))),
    }
}

fn extract_switchover_output(output: CommandOutputDto) -> Result<AcceptedResponse> {
    match output {
        CommandOutputDto::Switchover { output } => Ok(output),
        other => Err(HarnessError::message(format!(
            "expected `pgtm switchover request --json` output, observed command payload `{}`",
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

fn host_postgres_dsn(
    member: ClusterMember,
    port: u16,
    ca_cert_path: &Path,
    observer_cert_path: &Path,
    observer_key_path: &Path,
) -> String {
    [
        ("host", member.service_name().to_string()),
        ("hostaddr", "127.0.0.1".to_string()),
        ("port", port.to_string()),
        ("user", "postgres".to_string()),
        ("dbname", "postgres".to_string()),
        ("sslmode", "verify-full".to_string()),
        ("sslrootcert", ca_cert_path.display().to_string()),
        ("sslcert", observer_cert_path.display().to_string()),
        ("sslkey", observer_key_path.display().to_string()),
    ]
    .into_iter()
    .map(|(key, value)| format!("{key}={}", render_conninfo_value(value.as_str())))
    .collect::<Vec<_>>()
    .join(" ")
}

fn render_host_observer_config(
    member: ClusterMember,
    resolve_to: SocketAddr,
    ca_cert_path: &Path,
    read_token_path: &Path,
    admin_token_path: &Path,
    observer_cert_path: &Path,
    observer_key_path: &Path,
) -> String {
    format!(
        r#"[api]
base_url = "https://{}:{}"
expected_transport = "https"
resolve_to = "{resolve_to}"
auth = {{ type = "role_tokens", tokens = {{ read_token = {{ type = "file", path = "{}" }}, admin_token = {{ type = "file", path = "{}" }} }} }}
tls = {{ ca_cert = {{ path = "{}" }}, identity = {{ cert = {{ path = "{}" }}, key = {{ type = "file", path = "{}" }} }} }}

[postgres.tls]
ca_cert = {{ path = "{}" }}
identity = {{ cert = {{ path = "{}" }}, key = {{ type = "file", path = "{}" }} }}
"#,
        member.service_name(),
        resolve_to.port(),
        read_token_path.display(),
        admin_token_path.display(),
        ca_cert_path.display(),
        observer_cert_path.display(),
        observer_key_path.display(),
        ca_cert_path.display(),
        observer_cert_path.display(),
        observer_key_path.display(),
    )
}

#[cfg(test)]
mod tests {
    use super::host_postgres_dsn;
    use crate::support::topology::ClusterMember;
    use std::path::Path;

    #[test]
    fn host_postgres_dsn_quotes_tls_paths() {
        let dsn = host_postgres_dsn(
            ClusterMember::NodeA,
            5432,
            Path::new("/tmp/ca bundle.pem"),
            Path::new("/tmp/observer cert.pem"),
            Path::new("/tmp/observer key.pem"),
        );

        assert!(dsn.contains("sslrootcert='/tmp/ca bundle.pem'"));
        assert!(dsn.contains("sslcert='/tmp/observer cert.pem'"));
        assert!(dsn.contains("sslkey='/tmp/observer key.pem'"));
    }
}
