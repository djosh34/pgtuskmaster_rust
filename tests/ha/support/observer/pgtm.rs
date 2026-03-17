use std::{
    fs,
    future::Future,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use pgtuskmaster_rust::{
    api::{AcceptedResponse, NodeState},
    dcs::MemberPostgresView,
    ha::types::{AuthorityProjection, PublicationState},
    state::{MemberId, SwitchoverState},
};
use reqwest::{Certificate, Client, StatusCode, Url};

use crate::support::{
    docker::cli::DockerCli,
    error::{HarnessError, Result},
    topology::ClusterMember,
};

pub type ClusterStatusView = NodeState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimaryRoutingTarget {
    pub member: ClusterMember,
    pub dsn: String,
}

#[derive(Clone, Debug)]
struct SelectedSeed {
    state: ClusterStatusView,
}

#[derive(Clone, Debug)]
struct MemberApiClient {
    client: Client,
    base_url: Url,
    read_token: String,
    admin_token: String,
    ca_cert_path: PathBuf,
}

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

    pub fn state(&self) -> Result<ClusterStatusView> {
        self.select_seed().map(|seed| seed.state)
    }

    pub fn state_via_member(&self, member: ClusterMember) -> Result<ClusterStatusView> {
        let client = self.member_api_client(member)?;
        self.fetch_state(&client, member)
    }

    pub fn state_and_primary_target(&self) -> Result<(ClusterStatusView, PrimaryRoutingTarget)> {
        let (seed, primary) = self.select_seed_with_primary_view()?;
        Ok((seed.state, primary))
    }

    pub fn switchover_request_via_member(
        &self,
        member: ClusterMember,
        target: Option<ClusterMember>,
    ) -> Result<String> {
        let client = self.member_api_client(member)?;
        let request = match target {
            Some(target_member) => SwitchoverState::Specific(MemberId(
                target_member.service_name().to_string(),
            )),
            None => SwitchoverState::AnyHealthyReplica,
        };
        let url = client.base_url.join("switchover").map_err(|source| {
            HarnessError::message(format!(
                "building switchover URL for `{member}` failed: {source}"
            ))
        })?;
        let response = run_async(
            client
                .client
                .post(url)
                .bearer_auth(client.admin_token.as_str())
                .json(&request)
                .send(),
        )?
        .map_err(|source| {
            HarnessError::message(format!(
                "pgtm switchover request via `{member}` failed: {source}"
            ))
        })?;
        let accepted = decode_json_response::<AcceptedResponse>(
            response,
            StatusCode::ACCEPTED,
            format!("pgtm switchover request via `{member}`"),
        )?;
        serde_json::to_string(&accepted).map_err(|source| {
            HarnessError::message(format!("serializing switchover response failed: {source}"))
        })
    }

    fn select_seed(&self) -> Result<SelectedSeed> {
        let mut best_seed = None;
        let mut best_score = None;
        let mut errors = Vec::new();
        for member in ClusterMember::ALL {
            match self.state_via_member(member) {
                Ok(state) => {
                    let score = status_score(&state);
                    match best_score {
                        Some(previous) if previous >= score => {}
                        _ => {
                            best_score = Some(score);
                            best_seed = Some(SelectedSeed { state });
                        }
                    }
                }
                Err(err) => errors.push(format!("{}: {err}", member.service_name())),
            }
        }
        best_seed.ok_or_else(|| aggregate_seed_failure("pgtm status", &errors))
    }

    fn select_seed_with_primary_view(&self) -> Result<(SelectedSeed, PrimaryRoutingTarget)> {
        let mut best_seed = None;
        let mut best_score = None;
        let mut errors = Vec::new();

        for member in ClusterMember::ALL {
            let client = match self.member_api_client(member) {
                Ok(client) => client,
                Err(err) => {
                    errors.push(format!("{} client: {err}", member.service_name()));
                    continue;
                }
            };
            let state = match self.fetch_state(&client, member) {
                Ok(state) => state,
                Err(err) => {
                    errors.push(format!("{} status: {err}", member.service_name()));
                    continue;
                }
            };
            let primary = match self.primary_routing_target(&state, client.ca_cert_path.as_path()) {
                Ok(primary) => primary,
                Err(err) => {
                    errors.push(format!("{} primary: {err}", member.service_name()));
                    continue;
                }
            };

            if !status_matches_primary_target(&state, &primary) {
                errors.push(format!(
                    "{} inconsistent: status primary={:?}, primary target={:?}",
                    member.service_name(),
                    status_primary_member(&state),
                    Some(primary.member.service_name().to_string()),
                ));
                continue;
            }

            let score = status_score(&state);
            match best_score {
                Some(previous) if previous >= score => {}
                _ => {
                    best_score = Some(score);
                    best_seed = Some((SelectedSeed { state }, primary));
                }
            }
        }

        best_seed.ok_or_else(|| aggregate_seed_failure("pgtm status/primary consistency", &errors))
    }

    fn member_api_client(&self, member: ClusterMember) -> Result<MemberApiClient> {
        let ca_cert_path = self.ca_cert_path();
        let published_api_port = self.member_published_port(member, "8443/tcp")?;
        let ca_cert = Certificate::from_pem(
            fs::read(ca_cert_path.as_path())
                .map_err(|source| HarnessError::Io {
                    path: ca_cert_path.clone(),
                    source,
                })?
                .as_slice(),
        )
        .map_err(|source| {
            HarnessError::message(format!(
                "parsing HA CA certificate for `{member}` failed: {source}"
            ))
        })?;
        let client = Client::builder()
            .timeout(Duration::from_millis(5_000))
            .pool_max_idle_per_host(0)
            .add_root_certificate(ca_cert)
            .resolve(
                member.service_name(),
                SocketAddr::from(([127, 0, 0, 1], published_api_port)),
            )
            .build()
            .map_err(|source| {
                HarnessError::message(format!(
                    "building host-side HA API client for `{member}` failed: {source}"
                ))
            })?;
        let base_url = Url::parse(
            format!("https://{}:{published_api_port}/", member.service_name()).as_str(),
        )
        .map_err(|source| {
            HarnessError::message(format!(
                "building host API URL for `{member}` failed: {source}"
            ))
        })?;

        Ok(MemberApiClient {
            client,
            base_url,
            read_token: read_secret(self.read_token_path().as_path())?,
            admin_token: read_secret(self.admin_token_path().as_path())?,
            ca_cert_path,
        })
    }

    fn fetch_state(&self, client: &MemberApiClient, member: ClusterMember) -> Result<ClusterStatusView> {
        let url = client.base_url.join("state").map_err(|source| {
            HarnessError::message(format!(
                "building state URL for `{member}` failed: {source}"
            ))
        })?;
        let response = run_async(
            client
                .client
                .get(url)
                .bearer_auth(client.read_token.as_str())
                .send(),
        )?
        .map_err(|source| {
            HarnessError::message(format!("pgtm status via `{member}` failed: {source}"))
        })?;
        decode_json_response::<NodeState>(
            response,
            StatusCode::OK,
            format!("pgtm status via `{member}`"),
        )
    }

    fn primary_routing_target(
        &self,
        state: &ClusterStatusView,
        ca_cert_path: &Path,
    ) -> Result<PrimaryRoutingTarget> {
        let primary_member = status_primary_member(state)
            .as_deref()
            .map(ClusterMember::parse)
            .transpose()?
            .ok_or_else(|| {
                HarnessError::message(
                    "seed state does not currently expose an authoritative primary",
                )
            })?;
        let published_port = self.member_published_port(primary_member, "5432/tcp")?;
        Ok(PrimaryRoutingTarget {
            member: primary_member,
            dsn: host_postgres_dsn(
                primary_member,
                published_port,
                ca_cert_path,
                self.observer_cert_path().as_path(),
                self.observer_key_path().as_path(),
            ),
        })
    }

    fn member_published_port(&self, member: ClusterMember, port: &str) -> Result<u16> {
        let container_id = self.docker.compose_container_id(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            member.service_name(),
        )?;
        self.docker.published_host_port(container_id.as_str(), port)
    }

    fn ca_cert_path(&self) -> PathBuf {
        self.materialized_dir.join("configs/tls/ca.crt")
    }

    fn read_token_path(&self) -> PathBuf {
        self.materialized_dir.join("secrets/api-read-token")
    }

    fn admin_token_path(&self) -> PathBuf {
        self.materialized_dir.join("secrets/api-admin-token")
    }

    fn observer_cert_path(&self) -> PathBuf {
        self.materialized_dir.join("configs/tls/observer.crt")
    }

    fn observer_key_path(&self) -> PathBuf {
        self.materialized_dir.join("configs/tls/observer.key")
    }
}

fn decode_json_response<T>(
    response: reqwest::Response,
    expected_status: StatusCode,
    context: String,
) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let status = response.status();
    let body = run_async(response.text())?.map_err(|source| {
        HarnessError::message(format!("{context} response body read failed: {source}"))
    })?;
    if status != expected_status {
        return Err(HarnessError::message(format!(
            "{context} returned status {} with body {body}",
            status.as_u16()
        )));
    }
    serde_json::from_str(body.as_str()).map_err(|source| HarnessError::Json { context, source })
}

fn run_async<F>(future: F) -> Result<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|source| {
                HarnessError::message(format!(
                    "building helper tokio runtime for HA observer failed: {source}"
                ))
            })
            .map(|runtime| runtime.block_on(future));
        let _ = sender.send(result);
    });
    receiver.recv().map_err(|source| {
        HarnessError::message(format!(
            "waiting for HA observer async helper thread failed: {source}"
        ))
    })?
}

fn read_secret(path: &Path) -> Result<String> {
    let raw = fs::read_to_string(path).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(raw.trim().to_string())
}

fn host_postgres_dsn(
    member: ClusterMember,
    port: u16,
    ca_cert_path: &Path,
    observer_cert_path: &Path,
    observer_key_path: &Path,
) -> String {
    [
        format!("host={}", member.service_name()),
        "hostaddr=127.0.0.1".to_string(),
        format!("port={port}"),
        "user=postgres".to_string(),
        "dbname=postgres".to_string(),
        "sslmode=verify-full".to_string(),
        format!("sslrootcert={}", ca_cert_path.display()),
        format!("sslcert={}", observer_cert_path.display()),
        format!("sslkey={}", observer_key_path.display()),
    ]
    .join(" ")
}

fn aggregate_seed_failure(operation: &str, errors: &[String]) -> HarnessError {
    HarnessError::message(format!(
        "{operation} failed for every seed member:\n{}",
        errors.join("\n")
    ))
}

fn status_score(status: &ClusterStatusView) -> (usize, usize, usize, usize) {
    let reported_primary_count = status
        .dcs
        .members()
        .filter(|(_member_id, member)| {
            matches!(member.postgres(), MemberPostgresView::Primary { .. })
        })
        .count();
    let discovered_member_count = status.dcs.member_count();
    (
        usize::from(status.dcs.is_quorum()),
        usize::from(matches!(
            &status.ha.publication,
            PublicationState::Projected(AuthorityProjection::Primary(_))
        )),
        usize::from(status.dcs.is_quorum() && reported_primary_count == 1),
        discovered_member_count,
    )
}

fn status_matches_primary_target(
    status: &ClusterStatusView,
    primary: &PrimaryRoutingTarget,
) -> bool {
    status_primary_member(status).as_deref() == Some(primary.member.service_name())
}

fn status_primary_member(status: &ClusterStatusView) -> Option<String> {
    match &status.ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            Some(epoch.holder.0.clone())
        }
        PublicationState::Unknown
        | PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => None,
    }
}
