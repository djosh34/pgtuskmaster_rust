use std::{
    collections::BTreeMap,
    fs,
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use pgtuskmaster_rust::{
    api::NodeState,
    command::CommandOutputDto,
    pginfo::{
        conninfo::PgClientTls,
        state::{PgConnInfo, PgSslMode},
    },
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
    pub conninfo: PgConnInfo,
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
            conninfo: host_postgres_conninfo(
                member,
                published_port,
                ca_cert_path.as_path(),
                observer_cert_path.as_path(),
                observer_key_path.as_path(),
            )?,
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
        )?;
        match output {
            Ok(CommandOutputDto::Switchover { output }) => {
                serde_json::to_string(&output).map_err(|source| {
                    HarnessError::message(format!(
                        "serializing switchover response failed: {source}"
                    ))
                })
            }
            Ok(other) => Err(HarnessError::message(format!(
                "expected `pgtm switchover request --json` output, observed command payload `{}`",
                command_label(&other)
            ))),
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
        )
        .map_err(|err| err.to_string())?
        .and_then(|output| match output {
            CommandOutputDto::State { output } => Ok(output.state),
            other => Err(format!(
                "expected `pgtm status --json` output, observed command payload `{}`",
                command_label(&other)
            )),
        })
    }

    fn run_command_via_member(
        &self,
        member: ClusterMember,
        runtime_config: &Path,
        command_args: Vec<String>,
        context_label: &str,
    ) -> Result<std::result::Result<CommandOutputDto, String>> {
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
            None,
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
                Ok(Ok(dto))
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
        let config = build_host_observer_config(
            member,
            SocketAddr::from(([127, 0, 0, 1], published_api_port)),
            ca_cert_path.as_path(),
            read_token_path.as_path(),
            admin_token_path.as_path(),
            observer_cert_path.as_path(),
            observer_key_path.as_path(),
        );
        pgtuskmaster_test_support::runtime_config::validate_operator_config_contents(
            config.as_str(),
        )
        .map_err(|source| {
            HarnessError::message(format!(
                "rendered observer config for `{member}` failed validation: {source}"
            ))
        })?;
        fs::write(config_path.as_path(), config).map_err(|source| HarnessError::Io {
            path: config_path.clone(),
            source,
        })?;
        Ok(config_path)
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

fn host_postgres_conninfo(
    member: ClusterMember,
    port: u16,
    ca_cert_path: &Path,
    observer_cert_path: &Path,
    observer_key_path: &Path,
) -> Result<PgConnInfo> {
    Ok(PgConnInfo {
        route: pgtuskmaster_rust::state::PgRoute::tcp_hostaddr(
            member.service_name().to_string(),
            port,
            Some(Ipv4Addr::LOCALHOST.into()),
        )
        .map_err(HarnessError::message)?,
        user: "postgres".to_string(),
        dbname: "postgres".to_string(),
        application_name: None,
        connect_timeout_s: None,
        options: None,
        tls: PgClientTls {
            mode: PgSslMode::VerifyFull,
            root_cert: Some(ca_cert_path.to_path_buf()),
            client_cert: Some(observer_cert_path.to_path_buf()),
            client_key: Some(observer_key_path.to_path_buf()),
        },
    })
}

fn build_host_observer_config(
    member: ClusterMember,
    resolve_to: SocketAddr,
    ca_cert_path: &Path,
    read_token_path: &Path,
    admin_token_path: &Path,
    observer_cert_path: &Path,
    observer_key_path: &Path,
) -> String {
    let base_url = format!("https://{}:{}", member.service_name(), resolve_to.port());
    let resolve_to = resolve_to.to_string();
    let ca_cert = path_source(ca_cert_path);
    let read_token = path_source(read_token_path);
    let admin_token = path_source(admin_token_path);
    let observer_cert = path_source(observer_cert_path);
    let observer_key = path_source(observer_key_path);
    format!(
        r#"[api]
base_url = {base_url}
expected_transport = "https"
resolve_to = {resolve_to}

[api.auth]
type = "role_tokens"
read_token = {read_token}
admin_token = {admin_token}

[api.tls]
ca_cert = {ca_cert}

[api.tls.identity]
cert = {observer_cert}
key = {observer_key}

[postgres.tls]
ca_cert = {ca_cert}

[postgres.tls.identity]
cert = {observer_cert}
key = {observer_key}
"#,
        base_url = toml_string(base_url.as_str()),
        resolve_to = toml_string(resolve_to.as_str()),
        read_token = read_token,
        admin_token = admin_token,
        ca_cert = ca_cert,
        observer_cert = observer_cert,
        observer_key = observer_key,
    )
}

fn path_source(path: &Path) -> String {
    format!(
        "{{ path = {} }}",
        toml_string(path.display().to_string().as_str())
    )
}

fn toml_string(value: &str) -> String {
    toml::Value::String(value.to_string()).to_string()
}

#[cfg(test)]
mod tests {
    use super::{build_host_observer_config, host_postgres_conninfo};
    use crate::support::topology::ClusterMember;
    use std::{
        fs,
        net::SocketAddr,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| err.to_string())?
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "pgtm-observer-{label}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        Ok(dir)
    }

    #[test]
    fn host_postgres_conninfo_renders_tls_paths() -> Result<(), String> {
        let dsn = host_postgres_conninfo(
            ClusterMember::NodeA,
            5432,
            Path::new("/tmp/ca bundle.pem"),
            Path::new("/tmp/observer cert.pem"),
            Path::new("/tmp/observer key.pem"),
        )
        .map_err(|err| err.to_string())?
        .to_string();

        assert!(dsn.contains("sslrootcert='/tmp/ca bundle.pem'"));
        assert!(dsn.contains("sslcert='/tmp/observer cert.pem'"));
        assert!(dsn.contains("sslkey='/tmp/observer key.pem'"));
        assert!(dsn.contains("hostaddr=127.0.0.1"));
        Ok(())
    }

    #[test]
    fn host_observer_config_round_trips_through_toml() -> Result<(), String> {
        let dir = unique_test_dir("round-trip")?;
        let ca_path = dir.join("ca bundle.pem");
        let read_token_path = dir.join("read token");
        let admin_token_path = dir.join("admin token");
        let observer_cert_path = dir.join("observer cert.pem");
        let observer_key_path = dir.join("observer key.pem");
        for path in [
            &ca_path,
            &read_token_path,
            &admin_token_path,
            &observer_cert_path,
            &observer_key_path,
        ] {
            fs::write(path, "placeholder").map_err(|err| err.to_string())?;
        }
        let rendered = build_host_observer_config(
            ClusterMember::NodeB,
            SocketAddr::from(([127, 0, 0, 1], 18443)),
            ca_path.as_path(),
            read_token_path.as_path(),
            admin_token_path.as_path(),
            observer_cert_path.as_path(),
            observer_key_path.as_path(),
        );
        pgtuskmaster_test_support::runtime_config::validate_operator_config_contents(
            rendered.as_str(),
        )
        .map_err(|err| err.to_string())?;
        assert!(rendered.contains(r#"base_url = "https://node-b:18443""#));
        assert!(rendered.contains(r#"resolve_to = "127.0.0.1:18443""#));
        assert!(rendered.contains(r#"type = "role_tokens""#));
        assert!(rendered.contains(read_token_path.display().to_string().as_str()));
        assert!(rendered.contains(observer_key_path.display().to_string().as_str()));
        let _ = fs::remove_dir_all(dir);
        Ok(())
    }
}
