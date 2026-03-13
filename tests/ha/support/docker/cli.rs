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
    process::{self, CommandSpec},
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

#[derive(Clone, Debug, Deserialize)]
struct DockerInspectEntry {
    #[serde(rename = "NetworkSettings")]
    network_settings: Option<DockerNetworkSettings>,
    #[serde(rename = "State")]
    state: Option<DockerContainerState>,
}

#[derive(Clone, Debug, Deserialize)]
struct DockerNetworkSettings {
    #[serde(rename = "Ports")]
    ports: Option<BTreeMap<String, Option<Vec<DockerPortBinding>>>>,
    #[serde(rename = "Networks")]
    networks: Option<BTreeMap<String, DockerNetworkEndpoint>>,
}

#[derive(Clone, Debug, Deserialize)]
struct DockerPortBinding {
    #[serde(rename = "HostPort")]
    host_port: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct DockerNetworkEndpoint {
    #[serde(rename = "IPAddress")]
    ip_address: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct DockerContainerState {
    #[serde(rename = "Status")]
    status: Option<String>,
    #[serde(rename = "Health")]
    health: Option<DockerHealthState>,
}

#[derive(Clone, Debug, Deserialize)]
struct DockerHealthState {
    #[serde(rename = "Status")]
    status: Option<String>,
}

impl DockerCli {
    pub fn discover() -> Result<Self> {
        let settings = harness_settings()?;
        let candidate = settings
            .docker
            .executable_candidates
            .iter()
            .find(|path| path.exists())
            .ok_or_else(|| {
                HarnessError::message(
                    "docker binary was not found in tests/ha/harness.toml executable_candidates",
                )
            })?;
        process::ensure_absolute_executable(candidate.as_path())?;
        Ok(Self {
            executable: candidate.clone(),
        })
    }

    pub fn verify_daemon(&self) -> Result<()> {
        let _ = self
            .run(["info"], "checking docker daemon availability")
            .map_err(annotate_docker_daemon_error)?;
        let _ = self.run(
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
        let _ = self.run_in_dir(
            compose_file.parent().ok_or_else(|| {
                HarnessError::message(format!(
                    "compose file `{}` has no parent directory",
                    compose_file.display()
                ))
            })?,
            args,
            if services.is_empty() {
                "starting docker compose stack".to_string()
            } else {
                format!("starting docker compose services `{}`", services.join(", "))
            },
        )?;
        Ok(())
    }

    pub fn compose_down(&self, compose_file: &Path, project: &str) -> Result<()> {
        let _ = self.run_in_dir(
            compose_file.parent().ok_or_else(|| {
                HarnessError::message(format!(
                    "compose file `{}` has no parent directory",
                    compose_file.display()
                ))
            })?,
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

    pub fn compose_ps_entries(
        &self,
        compose_file: &Path,
        project: &str,
    ) -> Result<Vec<ComposePsEntry>> {
        let output = self.run_text_in_dir(
            compose_file.parent().ok_or_else(|| {
                HarnessError::message(format!(
                    "compose file `{}` has no parent directory",
                    compose_file.display()
                ))
            })?,
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

    pub fn compose_logs(&self, compose_file: &Path, project: &str) -> Result<String> {
        self.run_text_in_dir(
            compose_file.parent().ok_or_else(|| {
                HarnessError::message(format!(
                    "compose file `{}` has no parent directory",
                    compose_file.display()
                ))
            })?,
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
        self.run_text(
            ["inspect".to_string(), container.to_string()],
            format!("inspecting docker container `{container}`"),
        )
    }

    pub fn container_logs(&self, container: &str) -> Result<String> {
        self.run_text(
            [
                "logs".to_string(),
                "--timestamps".to_string(),
                container.to_string(),
            ],
            format!("capturing docker logs for container `{container}`"),
        )
    }

    pub fn kill_container(&self, container: &str) -> Result<()> {
        let _ = self.run(
            ["kill".to_string(), container.to_string()],
            format!("killing container `{container}`"),
        )?;
        Ok(())
    }

    pub fn start_container(&self, container: &str) -> Result<()> {
        let _ = self.run(
            ["start".to_string(), container.to_string()],
            format!("starting container `{container}`"),
        )?;
        Ok(())
    }

    pub fn remove_container_force(&self, container: &str) -> Result<()> {
        let _ = self.run(
            [
                "rm".to_string(),
                "--force".to_string(),
                container.to_string(),
            ],
            format!("removing container `{container}`"),
        )?;
        Ok(())
    }

    pub fn published_host_port(&self, container: &str, port: &str) -> Result<u16> {
        let inspect_entries = self.inspect_container_entries(container)?;
        let details = inspect_entries.first().ok_or_else(|| {
            HarnessError::message(format!(
                "docker inspect for `{container}` did not return a container object"
            ))
        })?;
        let port_bindings = details
            .network_settings
            .as_ref()
            .and_then(|value| value.ports.as_ref())
            .and_then(|value| value.get(port))
            .and_then(|value| value.as_ref())
            .ok_or_else(|| {
                HarnessError::message(format!(
                    "container `{container}` does not expose published port `{port}`"
                ))
            })?;
        let binding = port_bindings.first().ok_or_else(|| {
            HarnessError::message(format!(
                "container `{container}` has no host binding for `{port}`"
            ))
        })?;
        let host_port = binding.host_port.as_deref().ok_or_else(|| {
            HarnessError::message(format!(
                "container `{container}` binding for `{port}` is missing HostPort"
            ))
        })?;
        host_port.parse::<u16>().map_err(|err| {
            HarnessError::message(format!(
                "container `{container}` binding for `{port}` has invalid host port `{host_port}`: {err}"
            ))
        })
    }

    pub fn container_health_status(&self, container: &str) -> Result<Option<String>> {
        Ok(self
            .inspect_container_entries(container)?
            .first()
            .and_then(|entry| entry.state.as_ref())
            .and_then(|state| state.health.as_ref())
            .and_then(|health| health.status.clone()))
    }

    pub fn container_state_status(&self, container: &str) -> Result<String> {
        self.inspect_container_entries(container)?
            .first()
            .and_then(|entry| entry.state.as_ref())
            .and_then(|state| state.status.clone())
            .ok_or_else(|| {
                HarnessError::message(format!(
                    "container `{container}` inspect payload is missing State.Status"
                ))
            })
    }

    pub fn container_ipv4_address(&self, container: &str) -> Result<String> {
        let inspect_entries = self.inspect_container_entries(container)?;
        let details = inspect_entries.first().ok_or_else(|| {
            HarnessError::message(format!(
                "docker inspect for `{container}` did not return a container object"
            ))
        })?;
        details
            .network_settings
            .as_ref()
            .and_then(|value| value.networks.as_ref())
            .and_then(|value| {
                value
                    .values()
                    .filter_map(|endpoint| endpoint.ip_address.clone())
                    .find(|ip_address| !ip_address.is_empty())
            })
            .ok_or_else(|| {
                HarnessError::message(format!(
                    "container `{container}` does not expose a non-empty IPv4 address"
                ))
            })
    }

    pub fn exec(&self, container: &str, binary: &Path, args: &[&str]) -> Result<String> {
        self.exec_with_env(container, binary, args, &[])
    }

    pub fn exec_with_env(
        &self,
        container: &str,
        binary: &Path,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<String> {
        self.exec_with_options(container, None, binary, args, env)
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
        self.run_text(
            command,
            format!("executing `{}` in `{container}`", binary.display()),
        )
    }

    pub fn run_detached(&self, args: Vec<String>, context: impl Into<String>) -> Result<String> {
        self.run_text(args, context)
    }

    pub fn sleep_for_resource_cleanup(&self) {
        std::thread::sleep(Duration::from_secs(2));
    }

    fn run<I, S>(&self, args: I, context: impl Into<String>) -> Result<process::CommandOutput>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        let spec = self
            .apply_forwarded_environment(CommandSpec::new(self.executable.clone(), context.into()))
            .env("PATH", "")
            .args(args);
        process::run(spec)
    }

    fn run_text<I, S>(&self, args: I, context: impl Into<String>) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let context = context.into();
        self.run(args, context.clone())?.stdout_text(context)
    }

    fn run_in_dir<I, S>(
        &self,
        cwd: &Path,
        args: I,
        context: impl Into<String>,
    ) -> Result<process::CommandOutput>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        let spec = self
            .apply_forwarded_environment(CommandSpec::new(self.executable.clone(), context.into()))
            .cwd(cwd)
            .env("PATH", "")
            .args(args);
        process::run(spec)
    }

    fn apply_forwarded_environment(&self, spec: CommandSpec) -> CommandSpec {
        forwarded_environment()
            .into_iter()
            .fold(spec, |current, (key, value)| current.env(key, value))
    }

    fn run_text_in_dir<I, S>(
        &self,
        cwd: &Path,
        args: I,
        context: impl Into<String>,
    ) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let context = context.into();
        self.run_in_dir(cwd, args, context.clone())?
            .stdout_text(context)
    }

    fn inspect_container_entries(&self, container: &str) -> Result<Vec<DockerInspectEntry>> {
        let output = self.inspect_container(container)?;
        serde_json::from_str(output.as_str()).map_err(|source| HarnessError::Json {
            context: format!("parsing docker inspect json for `{container}`"),
            source,
        })
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
