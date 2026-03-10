use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use serde_json::Value;

use crate::support::{
    error::{HarnessError, Result},
    process::{self, CommandSpec},
};

const DEFAULT_DOCKER_BIN_CANDIDATES: [&str; 2] = ["/usr/bin/docker", "/usr/local/bin/docker"];

#[derive(Clone, Debug)]
pub struct DockerCli {
    executable: PathBuf,
}

impl DockerCli {
    pub fn discover() -> Result<Self> {
        if let Some(path) = std::env::var_os("PGTUSKMASTER_HA_DOCKER_BIN") {
            let candidate = PathBuf::from(path);
            process::ensure_absolute_executable(candidate.as_path())?;
            return Ok(Self {
                executable: candidate,
            });
        }

        let candidate = DEFAULT_DOCKER_BIN_CANDIDATES
            .iter()
            .map(PathBuf::from)
            .find(|path| path.exists())
            .ok_or_else(|| {
                HarnessError::message(
                    "docker binary was not found; set PGTUSKMASTER_HA_DOCKER_BIN to an absolute path or install docker at /usr/bin/docker",
                )
            })?;
        process::ensure_absolute_executable(candidate.as_path())?;
        Ok(Self {
            executable: candidate,
        })
    }

    pub fn verify_daemon(&self) -> Result<()> {
        let _ = self.run(["info"], "checking docker daemon availability")?;
        let _ = self.run(["compose", "version"], "checking docker compose plugin availability")?;
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
            "--build".to_string(),
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

    pub fn compose_ps_json(&self, compose_file: &Path, project: &str) -> Result<Value> {
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
        let entries = self.compose_ps_json(compose_file, project)?;
        let rows = entries.as_array().ok_or_else(|| {
            HarnessError::message("docker compose ps --format json did not return an array")
        })?;
        let matches = rows
            .iter()
            .filter_map(|entry| {
                let service_name = entry.get("Service")?.as_str()?;
                if service_name != service {
                    return None;
                }
                entry.get("ID")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
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
        let inspect = self.inspect_container_json(container)?;
        let details = inspect
            .as_array()
            .and_then(|entries| entries.first())
            .ok_or_else(|| {
                HarnessError::message(format!(
                    "docker inspect for `{container}` did not return a container object"
                ))
            })?;
        let port_bindings = details
            .get("NetworkSettings")
            .and_then(|value| value.get("Ports"))
            .and_then(|value| value.get(port))
            .and_then(Value::as_array)
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
        let host_port = binding
            .get("HostPort")
            .and_then(Value::as_str)
            .ok_or_else(|| {
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
        let inspect = self.inspect_container_json(container)?;
        Ok(inspect
            .as_array()
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("State"))
            .and_then(|state| state.get("Health"))
            .and_then(|health| health.get("Status"))
            .and_then(Value::as_str)
            .map(ToString::to_string))
    }

    pub fn container_state_status(&self, container: &str) -> Result<String> {
        let inspect = self.inspect_container_json(container)?;
        inspect
            .as_array()
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("State"))
            .and_then(|state| state.get("Status"))
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| {
                HarnessError::message(format!(
                    "container `{container}` inspect payload is missing State.Status"
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
        process::ensure_absolute_path(binary)?;
        let mut command = vec!["exec".to_string()];
        command.extend(
            env.iter()
                .flat_map(|(key, value)| ["--env".to_string(), format!("{key}={value}")]),
        );
        command.extend([container.to_string(), binary.display().to_string()]);
        command.extend(args.iter().map(|value| value.to_string()));
        self.run_text(command, format!("executing `{}` in `{container}`", binary.display()))
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
        let spec = CommandSpec::new(self.executable.clone(), context.into())
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
        let spec = CommandSpec::new(self.executable.clone(), context.into())
            .cwd(cwd)
            .env("PATH", "")
            .args(args);
        process::run(spec)
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
        self.run_in_dir(cwd, args, context.clone())?.stdout_text(context)
    }

    fn inspect_container_json(&self, container: &str) -> Result<Value> {
        let output = self.inspect_container(container)?;
        serde_json::from_str(output.as_str()).map_err(|source| HarnessError::Json {
            context: format!("parsing docker inspect json for `{container}`"),
            source,
        })
    }
}

fn parse_json_sequence(input: &str, context: String) -> Result<Value> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    if trimmed.starts_with('[') {
        return serde_json::from_str(trimmed).map_err(|source| HarnessError::Json {
            context,
            source,
        });
    }

    trimmed
        .lines()
        .map(|line| {
            serde_json::from_str::<Value>(line).map_err(|source| HarnessError::Json {
                context: context.clone(),
                source,
            })
        })
        .collect::<Result<Vec<_>>>()
        .map(Value::Array)
}
