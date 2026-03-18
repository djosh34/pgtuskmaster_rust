use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::support::{
    config::harness_settings,
    error::{HarnessError, Result},
    process,
};

#[derive(Clone, Debug)]
pub struct DockerCli {
    executable: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ComposePsEntry {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Service")]
    pub service: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug)]
struct ContainerInspectDetails {
    published_ports: BTreeMap<String, PublishedPort>,
    state_status: Option<String>,
    health_status: Option<String>,
    ipv4_address: Option<String>,
    network_gateway: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PublishedPort {
    Unpublished,
    MissingBinding,
    MissingHostPort,
    Bound(u16),
    InvalidHostPort(String),
}

#[derive(Clone, Debug, Deserialize)]
struct RawDockerInspectEntry {
    #[serde(rename = "NetworkSettings")]
    network_settings: Option<RawDockerNetworkSettings>,
    #[serde(rename = "State")]
    state: Option<RawDockerContainerState>,
}

#[derive(Clone, Debug, Deserialize)]
struct RawDockerNetworkSettings {
    #[serde(rename = "Ports")]
    ports: Option<BTreeMap<String, Option<Vec<RawDockerPortBinding>>>>,
    #[serde(rename = "Networks")]
    networks: Option<BTreeMap<String, RawDockerNetworkEndpoint>>,
}

#[derive(Clone, Debug, Deserialize)]
struct RawDockerPortBinding {
    #[serde(rename = "HostPort")]
    host_port: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct RawDockerNetworkEndpoint {
    #[serde(rename = "IPAddress")]
    ip_address: Option<String>,
    #[serde(rename = "Gateway")]
    gateway: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct RawDockerContainerState {
    #[serde(rename = "Status")]
    status: Option<String>,
    #[serde(rename = "Health")]
    health: Option<RawDockerHealthState>,
}

#[derive(Clone, Debug, Deserialize)]
struct RawDockerHealthState {
    #[serde(rename = "Status")]
    status: Option<String>,
}

impl ContainerInspectDetails {
    fn from_raw(raw: RawDockerInspectEntry) -> Self {
        let published_ports = raw
            .network_settings
            .as_ref()
            .and_then(|settings| settings.ports.as_ref())
            .map(|ports| {
                ports
                    .iter()
                    .map(|(container_port, bindings)| {
                        (
                            container_port.clone(),
                            PublishedPort::from_raw_bindings(bindings.as_ref()),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let state_status = raw.state.as_ref().and_then(|state| state.status.clone());
        let health_status = raw
            .state
            .as_ref()
            .and_then(|state| state.health.as_ref())
            .and_then(|health| health.status.clone());
        let ipv4_address = raw
            .network_settings
            .as_ref()
            .and_then(|settings| settings.networks.as_ref())
            .and_then(|networks| {
                networks
                    .values()
                    .filter_map(|endpoint| endpoint.ip_address.as_deref())
                    .find(|ip_address| !ip_address.is_empty())
                    .map(str::to_owned)
            });
        let network_gateway = raw
            .network_settings
            .as_ref()
            .and_then(|settings| settings.networks.as_ref())
            .and_then(|networks| {
                networks
                    .values()
                    .filter_map(|endpoint| endpoint.gateway.as_deref())
                    .find(|gateway| !gateway.is_empty())
                    .map(str::to_owned)
            });

        Self {
            published_ports,
            state_status,
            health_status,
            ipv4_address,
            network_gateway,
        }
    }

    fn published_host_port(&self, container: &str, port: &str) -> Result<u16> {
        match self.published_ports.get(port) {
            None | Some(PublishedPort::Unpublished) => Err(HarnessError::message(format!(
                "container `{container}` does not expose published port `{port}`"
            ))),
            Some(PublishedPort::MissingBinding) => Err(HarnessError::message(format!(
                "container `{container}` has no host binding for `{port}`"
            ))),
            Some(PublishedPort::MissingHostPort) => Err(HarnessError::message(format!(
                "container `{container}` binding for `{port}` is missing HostPort"
            ))),
            Some(PublishedPort::Bound(host_port)) => Ok(*host_port),
            Some(PublishedPort::InvalidHostPort(host_port)) => Err(HarnessError::message(format!(
                "container `{container}` binding for `{port}` has invalid host port `{host_port}`"
            ))),
        }
    }

    fn state_status(&self, container: &str) -> Result<String> {
        self.state_status.clone().ok_or_else(|| {
            HarnessError::message(format!(
                "container `{container}` inspect payload is missing State.Status"
            ))
        })
    }

    fn health_status(&self) -> Option<String> {
        self.health_status.clone()
    }

    fn ipv4_address(&self, container: &str) -> Result<String> {
        self.ipv4_address.clone().ok_or_else(|| {
            HarnessError::message(format!(
                "container `{container}` does not expose a non-empty IPv4 address"
            ))
        })
    }

    fn network_gateway(&self, container: &str) -> Result<String> {
        self.network_gateway.clone().ok_or_else(|| {
            HarnessError::message(format!(
                "container `{container}` does not expose a non-empty network gateway"
            ))
        })
    }
}

impl PublishedPort {
    fn from_raw_bindings(bindings: Option<&Vec<RawDockerPortBinding>>) -> Self {
        let Some(bindings) = bindings else {
            return Self::Unpublished;
        };
        let Some(binding) = bindings.first() else {
            return Self::MissingBinding;
        };
        let Some(host_port) = binding.host_port.as_deref() else {
            return Self::MissingHostPort;
        };
        match host_port.parse::<u16>() {
            Ok(host_port) => Self::Bound(host_port),
            Err(_) => Self::InvalidHostPort(host_port.to_string()),
        }
    }
}

impl DockerCli {
    pub fn discover() -> Result<Self> {
        Ok(Self {
            executable: harness_settings()?.docker_executable().to_path_buf(),
        })
    }

    pub fn verify_daemon(&self) -> Result<()> {
        let _ = self
            .run(None, ["info"], "checking docker daemon availability")
            .map_err(annotate_docker_daemon_error)?;
        let _ = self.run(
            None,
            ["compose", "version"],
            "checking docker compose plugin availability",
        )?;
        Ok(())
    }

    pub fn compose_up_services(
        &self,
        compose_file: &Path,
        project: &str,
        services: &[&str],
    ) -> Result<()> {
        let compose_dir = compose_file.parent().ok_or_else(|| {
            HarnessError::message(format!(
                "compose file `{}` has no parent directory",
                compose_file.display()
            ))
        })?;
        let mut args = vec![
            "compose".to_string(),
            "--project-name".to_string(),
            project.to_string(),
            "-f".to_string(),
            compose_file.display().to_string(),
            "up".to_string(),
            "--detach".to_string(),
        ];
        args.extend(services.iter().map(|service| (*service).to_string()));
        let context = if services.is_empty() {
            "starting docker compose stack".to_string()
        } else {
            format!("starting docker compose services `{}`", services.join(", "))
        };
        let mut attempts = 0_u8;
        loop {
            match self.run(Some(compose_dir), args.clone(), context.clone()) {
                Ok(_) => return Ok(()),
                Err(HarnessError::CommandFailed { stderr, .. })
                    if attempts < 2 && is_retryable_compose_network_race(stderr.as_str()) =>
                {
                    attempts = attempts.saturating_add(1);
                    std::thread::sleep(Duration::from_millis(250));
                }
                Err(err) => return Err(err),
            }
        }
    }

    pub fn compose_down(&self, compose_file: &Path, project: &str) -> Result<()> {
        let _ = self.run(
            Some(compose_file.parent().ok_or_else(|| {
                HarnessError::message(format!(
                    "compose file `{}` has no parent directory",
                    compose_file.display()
                ))
            })?),
            [
                "compose".to_string(),
                "--project-name".to_string(),
                project.to_string(),
                "-f".to_string(),
                compose_file.display().to_string(),
                "down".to_string(),
                "-v".to_string(),
                "--remove-orphans".to_string(),
            ],
            "stopping docker compose stack",
        )?;
        Ok(())
    }

    pub fn wait_for_network_absent(&self, network: &str) -> Result<()> {
        let mut attempts = 0_u8;
        while attempts < 40 {
            if !self.network_exists(network)? {
                return Ok(());
            }
            attempts = attempts.saturating_add(1);
            std::thread::sleep(Duration::from_millis(250));
        }

        Err(HarnessError::message(format!(
            "timed out waiting for docker network `{network}` to disappear"
        )))
    }

    pub fn compose_ps_entries(
        &self,
        compose_file: &Path,
        project: &str,
    ) -> Result<Vec<ComposePsEntry>> {
        let output = self.run(
            Some(compose_file.parent().ok_or_else(|| {
                HarnessError::message(format!(
                    "compose file `{}` has no parent directory",
                    compose_file.display()
                ))
            })?),
            [
                "compose".to_string(),
                "--project-name".to_string(),
                project.to_string(),
                "-f".to_string(),
                compose_file.display().to_string(),
                "ps".to_string(),
                "--all".to_string(),
                "--format".to_string(),
                "json".to_string(),
            ],
            "capturing docker compose ps as json",
        )?;
        parse_json_sequence(
            output.as_str(),
            "parsing docker compose ps json".to_string(),
        )
    }

    #[cfg(test)]
    pub(crate) fn fake_for_tests() -> Self {
        Self {
            executable: PathBuf::from("/usr/bin/docker"),
        }
    }

    pub fn compose_logs(&self, compose_file: &Path, project: &str) -> Result<String> {
        self.run(
            Some(compose_file.parent().ok_or_else(|| {
                HarnessError::message(format!(
                    "compose file `{}` has no parent directory",
                    compose_file.display()
                ))
            })?),
            [
                "compose".to_string(),
                "--project-name".to_string(),
                project.to_string(),
                "-f".to_string(),
                compose_file.display().to_string(),
                "logs".to_string(),
                "--no-color".to_string(),
                "--timestamps".to_string(),
            ],
            "capturing docker compose logs",
        )
    }

    pub fn compose_container_id(
        &self,
        compose_file: &Path,
        project: &str,
        service: &str,
    ) -> Result<String> {
        let matches = self
            .compose_ps_entries(compose_file, project)?
            .iter()
            .filter_map(|entry| {
                if entry.service != service {
                    return None;
                }
                Some(entry.id.clone())
            })
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [container_id] => Ok(container_id.clone()),
            [] => Err(HarnessError::message(format!(
                "docker compose service `{service}` has no container in project `{project}`"
            ))),
            _ => Err(HarnessError::message(format!(
                "docker compose service `{service}` resolved to multiple containers"
            ))),
        }
    }

    pub fn inspect_container(&self, container: &str) -> Result<String> {
        self.run(
            None,
            ["inspect".to_string(), container.to_string()],
            format!("inspecting docker container `{container}`"),
        )
    }

    pub fn kill_container(&self, container: &str) -> Result<()> {
        let _ = self.run(
            None,
            ["kill".to_string(), container.to_string()],
            format!("killing container `{container}`"),
        )?;
        Ok(())
    }

    pub fn start_container(&self, container: &str) -> Result<()> {
        let _ = self.run(
            None,
            ["start".to_string(), container.to_string()],
            format!("starting container `{container}`"),
        )?;
        Ok(())
    }

    pub fn remove_container_force(&self, container: &str) -> Result<()> {
        let _ = self.run(
            None,
            [
                "rm".to_string(),
                "--force".to_string(),
                container.to_string(),
            ],
            format!("removing container `{container}`"),
        )?;
        Ok(())
    }

    fn network_exists(&self, network: &str) -> Result<bool> {
        match self.run(
            None,
            [
                "network".to_string(),
                "inspect".to_string(),
                network.to_string(),
            ],
            format!("inspecting docker network `{network}`"),
        ) {
            Ok(_) => Ok(true),
            Err(HarnessError::CommandFailed { stderr, .. })
                if {
                    let normalized = stderr.to_ascii_lowercase();
                    normalized.contains("no such network")
                        || (normalized.contains("network") && normalized.contains("not found"))
                } =>
            {
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }

    pub fn published_host_port(&self, container: &str, port: &str) -> Result<u16> {
        self.inspect_container_details(container)?
            .published_host_port(container, port)
    }

    pub fn container_health_status(&self, container: &str) -> Result<Option<String>> {
        Ok(self.inspect_container_details(container)?.health_status())
    }

    pub fn container_state_status(&self, container: &str) -> Result<String> {
        self.inspect_container_details(container)?
            .state_status(container)
    }

    pub fn container_ipv4_address(&self, container: &str) -> Result<String> {
        self.inspect_container_details(container)?
            .ipv4_address(container)
    }

    pub fn container_network_gateway(&self, container: &str) -> Result<String> {
        self.inspect_container_details(container)?
            .network_gateway(container)
    }

    pub fn exec_as_user(
        &self,
        container: &str,
        user: &str,
        binary: &Path,
        args: &[&str],
    ) -> Result<String> {
        self.exec_with_options(container, Some(user), binary, args, &[])
    }

    fn exec_with_options(
        &self,
        container: &str,
        user: Option<&str>,
        binary: &Path,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<String> {
        process::ensure_absolute_path(binary)?;
        let mut command = vec!["exec".to_string()];
        if let Some(user) = user {
            command.extend(["--user".to_string(), user.to_string()]);
        }
        command.extend(
            env.iter()
                .flat_map(|(key, value)| ["--env".to_string(), format!("{key}={value}")]),
        );
        command.extend([container.to_string(), binary.display().to_string()]);
        command.extend(args.iter().map(|value| value.to_string()));
        self.run(
            None,
            command,
            format!("executing `{}` in `{container}`", binary.display()),
        )
    }

    pub fn run_detached(&self, args: Vec<String>, context: impl Into<String>) -> Result<String> {
        self.run(None, args, context)
    }

    pub fn sleep_for_resource_cleanup(&self) {
        std::thread::sleep(Duration::from_secs(2));
    }

    fn run<I, S>(&self, cwd: Option<&Path>, args: I, context: impl Into<String>) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut env = forwarded_environment();
        env.push(("PATH".to_string(), String::new()));
        process::run(self.executable.as_path(), cwd, context.into(), args, env)
    }

    fn inspect_container_details(&self, container: &str) -> Result<ContainerInspectDetails> {
        let output = self.inspect_container(container)?;
        let entries = serde_json::from_str::<Vec<RawDockerInspectEntry>>(output.as_str()).map_err(
            |source| HarnessError::Json {
                context: format!("parsing docker inspect json for `{container}`"),
                source,
            },
        )?;
        let Some(entry) = entries.into_iter().next() else {
            return Err(HarnessError::message(format!(
                "docker inspect for `{container}` did not return a container object"
            )));
        };
        Ok(ContainerInspectDetails::from_raw(entry))
    }
}

fn forwarded_environment() -> Vec<(String, String)> {
    [
        "DOCKER_CONFIG",
        "DOCKER_CONTEXT",
        "DOCKER_HOST",
        "HOME",
        "PGTM_CUCUMBER_TEST_IMAGE",
        "PGTM_CUCUMBER_TEST_RUN_ID",
        "PGTM_HA_SUBNET_MANIFEST",
        "XDG_CONFIG_HOME",
        "XDG_RUNTIME_DIR",
    ]
    .into_iter()
    .filter_map(|key| {
        std::env::var(key)
            .ok()
            .map(|value| (key.to_string(), value))
    })
    .collect::<Vec<_>>()
}

fn is_retryable_compose_network_race(stderr: &str) -> bool {
    stderr.contains("failed to set up container networking")
        && stderr.contains("network ")
        && stderr.contains(" not found")
}

fn parse_json_sequence(input: &str, context: String) -> Result<Vec<ComposePsEntry>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    if trimmed.starts_with('[') {
        return serde_json::from_str(trimmed)
            .map_err(|source| HarnessError::Json { context, source });
    }

    trimmed
        .lines()
        .map(|line| {
            serde_json::from_str::<ComposePsEntry>(line).map_err(|source| HarnessError::Json {
                context: context.clone(),
                source,
            })
        })
        .collect::<Result<Vec<_>>>()
}

fn annotate_docker_daemon_error(error: HarnessError) -> HarnessError {
    match error {
        HarnessError::CommandFailed {
            executable,
            context,
            status,
            stdout,
            stderr,
        } => HarnessError::CommandFailed {
            executable,
            context,
            status,
            stdout,
            stderr: docker_socket_permission_hint(stderr.as_str())
                .map(|hint| format!("{stderr}\nhint: {hint}"))
                .unwrap_or(stderr),
        },
        other => other,
    }
}

fn docker_socket_permission_hint(stderr: &str) -> Option<&'static str> {
    let normalized = stderr.to_ascii_lowercase();
    let is_permission_denied = normalized.contains("permission denied");
    let is_docker_socket_failure =
        normalized.contains("docker api") || normalized.contains("docker.sock");
    if is_permission_denied && is_docker_socket_failure {
        Some(
            "ensure this account can access /var/run/docker.sock (for example through the docker group), or point DOCKER_HOST at a reachable daemon",
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_json_sequence, ComposePsEntry, ContainerInspectDetails, HarnessError,
        RawDockerInspectEntry, Result,
    };

    fn parse_inspect_details(input: &str) -> Result<ContainerInspectDetails> {
        let entries =
            serde_json::from_str::<Vec<RawDockerInspectEntry>>(input).map_err(|source| {
                HarnessError::Json {
                    context: "parsing test docker inspect json".to_string(),
                    source,
                }
            })?;
        let Some(entry) = entries.into_iter().next() else {
            return Err(HarnessError::message(
                "expected test docker inspect json to contain one container object",
            ));
        };
        Ok(ContainerInspectDetails::from_raw(entry))
    }

    #[test]
    fn parse_json_sequence_accepts_json_lines() -> Result<()> {
        let entries = parse_json_sequence(
            "{\"ID\":\"c1\",\"Service\":\"node-a\"}\n{\"ID\":\"c2\",\"Service\":\"node-b\"}\n",
            "parsing compose ps json lines".to_string(),
        )?;

        assert_eq!(
            entries
                .iter()
                .map(|entry: &ComposePsEntry| entry.service.as_str())
                .collect::<Vec<_>>(),
            vec!["node-a", "node-b"]
        );
        Ok(())
    }

    #[test]
    fn inspect_details_normalize_state_network_and_port_access() -> Result<()> {
        let details = parse_inspect_details(
            r#"[{
  "NetworkSettings": {
    "Ports": {
      "5432/tcp": [{ "HostPort": "15432" }]
    },
    "Networks": {
      "default": {
        "IPAddress": "172.18.0.8",
        "Gateway": "172.18.0.1"
      }
    }
  },
  "State": {
    "Status": "running",
    "Health": { "Status": "healthy" }
  }
}]"#,
        )?;

        assert_eq!(details.published_host_port("node-a", "5432/tcp")?, 15432);
        assert_eq!(details.state_status("node-a")?, "running");
        assert_eq!(details.health_status(), Some("healthy".to_string()));
        assert_eq!(details.ipv4_address("node-a")?, "172.18.0.8");
        assert_eq!(details.network_gateway("node-a")?, "172.18.0.1");
        Ok(())
    }

    #[test]
    fn inspect_details_preserve_port_binding_failures_for_requested_port() -> Result<()> {
        let missing_binding = parse_inspect_details(
            r#"[{
  "NetworkSettings": {
    "Ports": {
      "5432/tcp": []
    }
  }
}]"#,
        )?;
        let missing_host_port = parse_inspect_details(
            r#"[{
  "NetworkSettings": {
    "Ports": {
      "5432/tcp": [{}]
    }
  }
}]"#,
        )?;
        let invalid_host_port = parse_inspect_details(
            r#"[{
  "NetworkSettings": {
    "Ports": {
      "5432/tcp": [{ "HostPort": "nope" }]
    }
  }
}]"#,
        )?;

        let missing_binding_error = match missing_binding.published_host_port("node-a", "5432/tcp")
        {
            Ok(host_port) => {
                return Err(HarnessError::message(format!(
                    "expected missing binding error, got host port `{host_port}`"
                )));
            }
            Err(error) => error,
        };
        assert!(missing_binding_error
            .to_string()
            .contains("has no host binding"));
        let missing_host_port_error =
            match missing_host_port.published_host_port("node-a", "5432/tcp") {
                Ok(host_port) => {
                    return Err(HarnessError::message(format!(
                        "expected missing HostPort error, got host port `{host_port}`"
                    )));
                }
                Err(error) => error,
            };
        assert!(missing_host_port_error
            .to_string()
            .contains("is missing HostPort"));
        let invalid_host_port_error =
            match invalid_host_port.published_host_port("node-a", "5432/tcp") {
                Ok(host_port) => {
                    return Err(HarnessError::message(format!(
                        "expected invalid HostPort error, got host port `{host_port}`"
                    )));
                }
                Err(error) => error,
            };
        assert!(invalid_host_port_error
            .to_string()
            .contains("has invalid host port `nope`"));
        Ok(())
    }
}
