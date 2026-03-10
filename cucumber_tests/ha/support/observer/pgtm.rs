use std::path::Path;

use serde::Deserialize;

use crate::support::{
    docker::cli::DockerCli,
    error::{HarnessError, Result},
};

const PGTM_BIN: &str = "/usr/local/bin/pgtm";

#[derive(Clone, Debug)]
pub struct PgtmObserver {
    docker: DockerCli,
    observer_container: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClusterStatusView {
    pub queried_via: QueryOrigin,
    pub sampled_member_count: usize,
    pub discovered_member_count: usize,
    pub warnings: Vec<ClusterWarning>,
    pub nodes: Vec<ClusterNodeView>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QueryOrigin {
    pub member_id: String,
    pub api_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClusterWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClusterNodeView {
    pub member_id: String,
    pub sampled: bool,
    pub role: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionView {
    pub targets: Vec<ConnectionTarget>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionTarget {
    pub member_id: String,
    pub dsn: String,
}

impl PgtmObserver {
    pub fn new(docker: DockerCli, observer_container: String) -> Self {
        Self {
            docker,
            observer_container,
        }
    }

    pub fn status(&self) -> Result<ClusterStatusView> {
        self.try_configs(|config| {
            let output = self.run(config, &["status", "--json"])?;
            parse_json(output.as_str(), format!("pgtm status via {}", config.display()))
        })
    }

    pub fn debug_verbose(&self) -> Result<serde_json::Value> {
        self.try_configs(|config| {
            let output = self.run(config, &["debug", "verbose", "--json"])?;
            parse_json(
                output.as_str(),
                format!("pgtm debug verbose via {}", config.display()),
            )
        })
    }

    pub fn primary_tls_json(&self) -> Result<ConnectionView> {
        self.try_configs(|config| {
            let output = self.run(config, &["primary", "--json", "--tls"])?;
            parse_json(
                output.as_str(),
                format!("pgtm primary --tls via {}", config.display()),
            )
        })
    }

    pub fn replicas_tls_json(&self) -> Result<ConnectionView> {
        self.try_configs(|config| {
            let output = self.run(config, &["replicas", "--json", "--tls"])?;
            parse_json(
                output.as_str(),
                format!("pgtm replicas --tls via {}", config.display()),
            )
        })
    }

    fn run(&self, config: &Path, args: &[&str]) -> Result<String> {
        let mut all_args = vec!["-c", config.to_str().ok_or_else(|| {
            HarnessError::message(format!("observer config path is not valid utf-8: {}", config.display()))
        })?];
        all_args.extend(args.iter().copied());
        self.docker.exec(
            self.observer_container.as_str(),
            Path::new(PGTM_BIN),
            all_args.as_slice(),
        )
    }

    fn try_configs<T, F>(&self, mut attempt: F) -> Result<T>
    where
        F: FnMut(&Path) -> Result<T>,
    {
        let configs = [
            Path::new("/etc/pgtuskmaster/observer/node-a.toml"),
            Path::new("/etc/pgtuskmaster/observer/node-b.toml"),
            Path::new("/etc/pgtuskmaster/observer/node-c.toml"),
        ];
        let mut errors = Vec::new();
        for config in configs {
            match attempt(config) {
                Ok(value) => return Ok(value),
                Err(err) => errors.push(format!("{}: {err}", config.display())),
            }
        }
        Err(HarnessError::message(format!(
            "all observer seeds failed:\n{}",
            errors.join("\n")
        )))
    }
}

fn parse_json<T>(input: &str, context: impl Into<String>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(input).map_err(|source| HarnessError::Json {
        context: context.into(),
        source,
    })
}
