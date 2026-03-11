use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterStatusView {
    pub queried_via: QueryOrigin,
    pub sampled_member_count: usize,
    pub discovered_member_count: usize,
    pub warnings: Vec<ClusterWarning>,
    pub nodes: Vec<ClusterNodeView>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryOrigin {
    pub member_id: String,
    pub api_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterNodeView {
    pub member_id: String,
    pub sampled: bool,
    pub role: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionView {
    pub targets: Vec<ConnectionTarget>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        let (statuses, errors) = self.collect_from_configs(|config| {
            let output = self.run(config, &["status", "--json"])?;
            parse_json(
                output.as_str(),
                format!("pgtm status via {}", config.display()),
            )
        });
        statuses
            .into_iter()
            .max_by_key(status_score)
            .ok_or_else(|| aggregate_seed_failure("pgtm status", &errors))
    }

    pub fn status_via_member(&self, member_id: &str) -> Result<ClusterStatusView> {
        let config = config_path(member_id)?;
        let output = self.run(config, &["status", "--json"])?;
        parse_json(
            output.as_str(),
            format!("pgtm status via {}", config.display()),
        )
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

    pub fn debug_verbose_via_member(&self, member_id: &str) -> Result<serde_json::Value> {
        let config = config_path(member_id)?;
        let output = self.run(config, &["debug", "verbose", "--json"])?;
        parse_json(
            output.as_str(),
            format!("pgtm debug verbose via {}", config.display()),
        )
    }

    pub fn primary_tls_json(&self) -> Result<ConnectionView> {
        let (views, errors) = self.collect_from_configs(|config| {
            let output = self.run(config, &["primary", "--json", "--tls"])?;
            parse_json(
                output.as_str(),
                format!("pgtm primary --tls via {}", config.display()),
            )
        });
        aggregate_connection_views("pgtm primary --tls", views, &errors)
    }

    pub fn primary_tls_json_via_member(&self, member_id: &str) -> Result<ConnectionView> {
        let config = config_path(member_id)?;
        let output = self.run(config, &["primary", "--json", "--tls"])?;
        parse_json(
            output.as_str(),
            format!("pgtm primary --tls via {}", config.display()),
        )
    }

    pub fn replicas_tls_json(&self) -> Result<ConnectionView> {
        let (views, errors) = self.collect_from_configs(|config| {
            let output = self.run(config, &["replicas", "--json", "--tls"])?;
            parse_json(
                output.as_str(),
                format!("pgtm replicas --tls via {}", config.display()),
            )
        });
        aggregate_connection_views("pgtm replicas --tls", views, &errors)
    }

    pub fn switchover_request_via_member(
        &self,
        member_id: &str,
        target: Option<&str>,
    ) -> Result<String> {
        let config = config_path(member_id)?;
        let args = match target {
            Some(target_member_id) => vec![
                "--json",
                "switchover",
                "request",
                "--switchover-to",
                target_member_id,
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

    fn try_configs<T, F>(&self, attempt: F) -> Result<T>
    where
        F: FnMut(&Path) -> Result<T>,
    {
        let (values, errors) = self.collect_from_configs(attempt);
        match values.into_iter().next() {
            Some(value) => Ok(value),
            None => Err(aggregate_seed_failure("observer request", &errors)),
        }
    }

    fn collect_from_configs<T, F>(&self, mut attempt: F) -> (Vec<T>, Vec<String>)
    where
        F: FnMut(&Path) -> Result<T>,
    {
        let mut values = Vec::new();
        let mut errors = Vec::new();
        for config in config_paths() {
            match attempt(config) {
                Ok(value) => values.push(value),
                Err(err) => errors.push(format!("{}: {err}", config.display())),
            }
        }
        (values, errors)
    }
}

fn config_paths() -> [&'static Path; 3] {
    [
        Path::new("/etc/pgtuskmaster/observer/node-a.toml"),
        Path::new("/etc/pgtuskmaster/observer/node-b.toml"),
        Path::new("/etc/pgtuskmaster/observer/node-c.toml"),
    ]
}

fn config_path(member_id: &str) -> Result<&'static Path> {
    match member_id {
        "node-a" => Ok(Path::new("/etc/pgtuskmaster/observer/node-a.toml")),
        "node-b" => Ok(Path::new("/etc/pgtuskmaster/observer/node-b.toml")),
        "node-c" => Ok(Path::new("/etc/pgtuskmaster/observer/node-c.toml")),
        _ => Err(HarnessError::message(format!(
            "no observer seed config exists for member `{member_id}`"
        ))),
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

fn aggregate_seed_failure(operation: &str, errors: &[String]) -> HarnessError {
    HarnessError::message(format!(
        "{operation} failed for every observer seed:\n{}",
        errors.join("\n")
    ))
}

fn aggregate_connection_views(
    operation: &str,
    views: Vec<ConnectionView>,
    errors: &[String],
) -> Result<ConnectionView> {
    if views.is_empty() {
        return Err(aggregate_seed_failure(operation, errors));
    }

    let mut targets = BTreeMap::new();
    for view in views {
        for target in view.targets {
            targets.insert(target.member_id.clone(), target);
        }
    }
    Ok(ConnectionView {
        targets: targets.into_values().collect::<Vec<_>>(),
    })
}

fn status_score(status: &ClusterStatusView) -> (usize, usize, usize) {
    let sampled_primaries = status
        .nodes
        .iter()
        .filter(|node| node.sampled && node.role == "primary")
        .count();
    let known_roles = status
        .nodes
        .iter()
        .filter(|node| node.sampled && node.role != "unknown")
        .count();
    (
        status.sampled_member_count,
        usize::from(sampled_primaries == 1),
        known_roles,
    )
}
