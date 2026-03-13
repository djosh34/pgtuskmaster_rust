use std::{collections::BTreeMap, path::Path};

use pgtuskmaster_rust::{
    api::NodeState,
    dcs::state::{DcsTrust, MemberPostgresView},
    ha::types::AuthorityView,
};
use serde::de::DeserializeOwned;

pub use pgtuskmaster_rust::cli::connect::{ConnectionTarget, ConnectionView};

use crate::support::{
    docker::cli::DockerCli,
    error::{HarnessError, Result},
};

const PGTM_BIN: &str = "/usr/local/bin/pgtm";

pub type ClusterStatusView = NodeState;

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

    pub fn state_via_member(&self, member_id: &str) -> Result<ClusterStatusView> {
        let config = config_path(member_id)?;
        let output = self.run(config, &["status", "--json"])?;
        parse_json(
            output.as_str(),
            format!("pgtm status via {}", config.display()),
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
    T: DeserializeOwned,
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
    let mut view_iter = views.into_iter();
    let first_view = view_iter
        .next()
        .ok_or_else(|| aggregate_seed_failure(operation, errors))?;
    let mut targets = BTreeMap::new();
    for view in std::iter::once(first_view.clone()).chain(view_iter) {
        for target in view.targets {
            targets.insert(target.member_id.clone(), target);
        }
    }
    Ok(ConnectionView {
        cluster_name: first_view.cluster_name,
        scope: first_view.scope,
        kind: first_view.kind,
        tls: first_view.tls,
        discovered_member_count: first_view.discovered_member_count,
        warnings: first_view.warnings,
        targets: targets.into_values().collect::<Vec<_>>(),
    })
}

fn status_score(status: &ClusterStatusView) -> (usize, usize, usize, usize) {
    let reported_primary_count = status
        .dcs
        .cache
        .member_slots
        .values()
        .filter(|member| matches!(&member.postgres, MemberPostgresView::Primary(_)))
        .count();
    (
        status.dcs.cache.member_slots.len(),
        usize::from(status.dcs.trust == DcsTrust::FullQuorum),
        usize::from(matches!(
            &status.ha.publication.authority,
            AuthorityView::Primary { .. }
        )),
        usize::from(reported_primary_count == 1),
    )
}
