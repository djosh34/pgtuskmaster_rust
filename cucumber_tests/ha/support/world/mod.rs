use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use cucumber::World;

use crate::support::{
    binary_paths,
    docker::{cli::DockerCli, ryuk::RyukGuard},
    error::{HarnessError, Result},
    feature_metadata,
    givens::THREE_NODE_PLAIN,
    observer::{
        pgtm::{ClusterStatusView, PgtmObserver},
        sql::SqlObserver,
    },
    register_run,
    timeouts::TimeoutModel,
};

#[derive(Debug, Default, World)]
pub struct HaWorld {
    pub harness: Option<Arc<Mutex<HarnessShared>>>,
}

impl HaWorld {
    pub fn reset(&mut self) {
        self.harness = None;
    }

    pub fn harness(&self) -> Result<Arc<Mutex<HarnessShared>>> {
        self.harness.clone().ok_or_else(|| {
            HarnessError::message("scenario harness has not been initialized")
        })
    }

    pub fn set_harness(&mut self, harness: Arc<Mutex<HarnessShared>>) {
        self.harness = Some(harness);
    }
}

#[derive(Debug)]
pub struct HarnessShared {
    pub run_id: String,
    pub feature_name: String,
    pub given_name: String,
    pub run_dir: PathBuf,
    pub source_copy_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub compose_file: PathBuf,
    pub compose_project: String,
    pub docker: DockerCli,
    pub ryuk: Option<RyukGuard>,
    pub observer_container: String,
    pub postgres_password: String,
    pub timeouts: TimeoutModel,
    pub killed_node: Option<String>,
    pub new_primary: Option<String>,
    pub proof_token: Option<String>,
    cleaned_up: bool,
}

impl HarnessShared {
    pub async fn initialize(given_name: &str) -> Result<Arc<Mutex<Self>>> {
        if given_name != THREE_NODE_PLAIN {
            return Err(HarnessError::message(format!(
                "unsupported given `{given_name}`; only `{THREE_NODE_PLAIN}` is implemented"
            )));
        }

        let feature = feature_metadata()?;
        let docker = DockerCli::discover()?;
        docker.verify_daemon()?;

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given_root = repo_root.join("cucumber_tests/ha/givens").join(THREE_NODE_PLAIN);
        let run_id = build_run_id(feature.feature_name.as_str())?;
        let compose_project = build_compose_project(feature.feature_name.as_str(), run_id.as_str());
        let run_dir = repo_root
            .join("cucumber_tests/ha/runs")
            .join(feature.feature_name.as_str())
            .join(run_id.as_str());
        let source_copy_dir = run_dir.join("source-copy");
        let artifacts_dir = run_dir.join("artifacts");
        create_dir_all(run_dir.as_path())?;
        create_dir_all(source_copy_dir.as_path())?;
        create_dir_all(artifacts_dir.as_path())?;
        copy_directory(given_root.as_path(), source_copy_dir.as_path())?;
        copy_test_binaries(source_copy_dir.as_path())?;

        let compose_file = source_copy_dir.join("compose.yml");
        let timeouts = TimeoutModel::from_runtime_config(
            source_copy_dir.join("configs/node-a/runtime.toml").as_path(),
        )?;
        let postgres_password = read_secret(
            source_copy_dir.join("secrets/postgres-superuser-password").as_path(),
        )?;
        let ryuk = RyukGuard::start(docker.clone(), compose_project.as_str())?;
        docker.compose_up_services(compose_file.as_path(), compose_project.as_str(), &["etcd"])?;
        docker.compose_up_services(
            compose_file.as_path(),
            compose_project.as_str(),
            &["observer"],
        )?;
        let observer_container = docker.compose_container_id(
            compose_file.as_path(),
            compose_project.as_str(),
            "observer",
        )?;

        let shared = Arc::new(Mutex::new(Self {
            run_id: run_id.clone(),
            feature_name: feature.feature_name.clone(),
            given_name: given_name.to_string(),
            run_dir,
            source_copy_dir,
            artifacts_dir,
            compose_file,
            compose_project,
            docker,
            ryuk: Some(ryuk),
            observer_container,
            postgres_password,
            timeouts,
            killed_node: None,
            new_primary: None,
            proof_token: None,
            cleaned_up: false,
        }));
        if let Err(err) = Self::bootstrap_cluster(shared.clone()).await {
            let cleanup_error = {
                let mut guard = lock_shared(shared.as_ref())?;
                guard.cleanup().err()
            };
            return match cleanup_error {
                None => Err(err),
                Some(cleanup) => Err(HarnessError::message(format!(
                    "{err}\ncleanup after bootstrap failure also failed: {cleanup}"
                ))),
            };
        }
        register_run(run_id, shared.clone())?;
        Ok(shared)
    }

    pub fn observer(&self) -> PgtmObserver {
        PgtmObserver::new(self.docker.clone(), self.observer_container.clone())
    }

    pub fn sql(&self) -> SqlObserver {
        SqlObserver::new(
            self.docker.clone(),
            self.observer_container.clone(),
            self.postgres_password.clone(),
        )
    }

    pub fn service_container_id(&self, service: &str) -> Result<String> {
        self.docker.compose_container_id(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            service,
        )
    }

    async fn bootstrap_cluster(shared: Arc<Mutex<Self>>) -> Result<()> {
        Self::wait_for_service_health(shared.clone(), "etcd").await?;
        {
            let guard = lock_shared(shared.as_ref())?;
            guard.docker.compose_up_services(
                guard.compose_file.as_path(),
                guard.compose_project.as_str(),
                &["node-b"],
            )?;
        }
        Self::wait_for_seed_primary(shared.clone()).await?;
        let guard = lock_shared(shared.as_ref())?;
        guard.docker.compose_up_services(
            guard.compose_file.as_path(),
            guard.compose_project.as_str(),
            &["node-a", "node-c"],
        )
    }

    async fn wait_for_service_health(shared: Arc<Mutex<Self>>, service: &str) -> Result<()> {
        let deadline = {
            let guard = lock_shared(shared.as_ref())?;
            Instant::now() + guard.timeouts.startup_deadline
        };
        let poll_interval = {
            let guard = lock_shared(shared.as_ref())?;
            guard.timeouts.poll_interval
        };

        let mut last_error = None;
        while Instant::now() < deadline {
            let result: Result<()> = {
                let guard = lock_shared(shared.as_ref())?;
                match guard
                    .service_container_id(service)
                    .and_then(|container_id| guard.docker.container_health_status(container_id.as_str()))
                {
                    Ok(Some(status)) if status == "healthy" => Ok(()),
                    Ok(Some(status)) => Err(HarnessError::message(format!(
                        "service `{service}` health is `{status}`"
                    ))),
                    Ok(None) => Err(HarnessError::message(format!(
                        "service `{service}` does not expose a docker health status"
                    ))),
                    Err(err) => Err(err),
                }
            };
            match result {
                Ok(()) => return Ok(()),
                Err(err) => last_error = Some(err.to_string()),
            }
            tokio::time::sleep(poll_interval).await;
        }

        Err(HarnessError::message(format!(
            "timed out waiting for service `{service}` to become healthy; last observed error: {}",
            last_error.unwrap_or_else(|| "no health state was observed".to_string())
        )))
    }

    async fn wait_for_seed_primary(shared: Arc<Mutex<Self>>) -> Result<()> {
        let deadline = {
            let guard = lock_shared(shared.as_ref())?;
            Instant::now() + guard.timeouts.startup_deadline
        };
        let poll_interval = {
            let guard = lock_shared(shared.as_ref())?;
            guard.timeouts.poll_interval
        };

        let mut last_error = None;
        while Instant::now() < deadline {
            let result: Result<()> = {
                let guard = lock_shared(shared.as_ref())?;
                match guard.observer().status() {
                    Ok(status) => validate_seed_primary(&status),
                    Err(err) => Err(err),
                }
            };
            match result {
                Ok(()) => return Ok(()),
                Err(err) => last_error = Some(err.to_string()),
            }
            tokio::time::sleep(poll_interval).await;
        }

        Err(HarnessError::message(format!(
            "timed out waiting for bootstrap primary before starting replicas; last observed error: {}",
            last_error.unwrap_or_else(|| "no status was observed".to_string())
        )))
    }

    pub fn cleanup(&mut self) -> Result<()> {
        if self.cleaned_up {
            return Ok(());
        }
        self.cleaned_up = true;

        let mut failures = Vec::new();
        if let Err(err) = self.capture_artifacts() {
            failures.push(format!("artifact capture failed: {err}"));
        }
        if let Err(err) = self
            .docker
            .compose_down(self.compose_file.as_path(), self.compose_project.as_str())
        {
            failures.push(format!("docker compose down failed: {err}"));
        }
        if let Some(ryuk) = self.ryuk.as_mut() {
            if let Err(err) = ryuk.close() {
                failures.push(format!("ryuk cleanup failed: {err}"));
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(HarnessError::message(failures.join("\n")))
        }
    }

    fn capture_artifacts(&self) -> Result<()> {
        write_text_file(
            self.artifacts_dir.join("compose-ps.json").as_path(),
            serde_json::to_string_pretty(
                &self
                    .docker
                    .compose_ps_json(self.compose_file.as_path(), self.compose_project.as_str())?,
            )
            .map_err(|source| HarnessError::Json {
                context: "serializing docker compose ps json".to_string(),
                source,
            })?
            .as_str(),
        )?;
        write_text_file(
            self.artifacts_dir.join("compose-logs.txt").as_path(),
            self.docker
                .compose_logs(self.compose_file.as_path(), self.compose_project.as_str())?
                .as_str(),
        )?;
        write_text_file(
            self.artifacts_dir.join("run-metadata.json").as_path(),
            serde_json::to_string_pretty(&serde_json::json!({
                "feature_name": self.feature_name,
                "given_name": self.given_name,
                "run_id": self.run_id,
                "run_dir": self.run_dir,
                "source_copy_dir": self.source_copy_dir,
                "artifacts_dir": self.artifacts_dir,
                "compose_project": self.compose_project,
            }))
            .map_err(|source| HarnessError::Json {
                context: "serializing run metadata".to_string(),
                source,
            })?
            .as_str(),
        )?;
        if let Ok(debug) = self.observer().debug_verbose() {
            write_text_file(
                self.artifacts_dir.join("observer-debug-verbose.json").as_path(),
                serde_json::to_string_pretty(&debug)
                    .map_err(|source| HarnessError::Json {
                        context: "serializing observer debug verbose payload".to_string(),
                        source,
                    })?
                    .as_str(),
            )?;
        }

        for service in ["observer", "node-a", "node-b", "node-c", "etcd"] {
            if let Ok(container_id) = self.service_container_id(service) {
                if let Ok(inspect) = self.docker.inspect_container(container_id.as_str()) {
                    let artifact = self
                        .artifacts_dir
                        .join(format!("inspect-{service}.json"));
                    let _ = write_text_file(artifact.as_path(), inspect.as_str());
                }
            }
        }
        Ok(())
    }
}

fn copy_test_binaries(source_copy_dir: &Path) -> Result<()> {
    let binaries = binary_paths()?;
    let destination_root = source_copy_dir.join("docker_files/bin");
    create_dir_all(destination_root.as_path())?;
    copy_file(
        binaries.pgtuskmaster.as_path(),
        destination_root.join("pgtuskmaster").as_path(),
    )?;
    copy_file(
        binaries.pgtm.as_path(),
        destination_root.join("pgtm").as_path(),
    )?;
    Ok(())
}

fn build_run_id(feature_name: &str) -> Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))?;
    Ok(format!(
        "{}-{}-{}",
        sanitize(feature_name),
        timestamp.as_millis(),
        std::process::id()
    ))
}

fn build_compose_project(feature_name: &str, run_id: &str) -> String {
    let feature = sanitize(feature_name);
    let run = sanitize(run_id);
    format!("ha-{}-{}", feature, run)
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
}

fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn copy_file(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|source| HarnessError::Io {
            path: to.to_path_buf(),
            source,
        })
}

fn read_secret(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let secret = content.trim().to_string();
    if secret.is_empty() {
        return Err(HarnessError::message(format!(
            "secret file `{}` was empty",
            path.display()
        )));
    }
    Ok(secret)
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn copy_directory(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        return Err(HarnessError::message(format!(
            "source directory does not exist: {}",
            from.display()
        )));
    }

    let mut directories = vec![(from.to_path_buf(), to.to_path_buf())];
    while let Some((current_from, current_to)) = directories.pop() {
        create_dir_all(current_to.as_path())?;
        for entry in fs::read_dir(current_from.as_path()).map_err(|source| HarnessError::Io {
            path: current_from.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| HarnessError::Io {
                path: current_from.clone(),
                source,
            })?;
            let source_path = entry.path();
            let destination_path = current_to.join(entry.file_name());
            if source_path.is_dir() {
                directories.push((source_path, destination_path));
            } else {
                copy_file(source_path.as_path(), destination_path.as_path())?;
            }
        }
    }
    Ok(())
}

fn lock_shared(shared: &Mutex<HarnessShared>) -> Result<std::sync::MutexGuard<'_, HarnessShared>> {
    shared
        .lock()
        .map_err(|_| HarnessError::message("harness mutex was poisoned"))
}

fn validate_seed_primary(status: &ClusterStatusView) -> Result<()> {
    if status.sampled_member_count != 1 {
        return Err(HarnessError::message(format!(
            "expected exactly one sampled member during bootstrap, observed {}; warnings={}",
            status.sampled_member_count,
            format_bootstrap_warnings(status),
        )));
    }

    let primaries = status
        .nodes
        .iter()
        .filter(|node| node.sampled && node.role == "primary")
        .map(|node| node.member_id.as_str())
        .collect::<Vec<_>>();
    match primaries.as_slice() {
        ["node-b"] => Ok(()),
        [] => Err(HarnessError::message(format!(
            "bootstrap status has no sampled primary; queried via {} and warnings={}",
            status.queried_via.member_id,
            format_bootstrap_warnings(status),
        ))),
        [primary] => Err(HarnessError::message(format!(
            "expected node-b to bootstrap as the seed primary, observed `{primary}`"
        ))),
        many => Err(HarnessError::message(format!(
            "bootstrap status has multiple sampled primaries: {}",
            many.join(", ")
        ))),
    }
}

fn format_bootstrap_warnings(status: &ClusterStatusView) -> String {
    if status.warnings.is_empty() {
        "none".to_string()
    } else {
        status
            .warnings
            .iter()
            .map(|warning| format!("{}:{}", warning.code, warning.message))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
