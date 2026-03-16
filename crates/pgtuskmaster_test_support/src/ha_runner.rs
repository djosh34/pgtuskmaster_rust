use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use pgtuskmaster_rust::{
    api::{AcceptedResponse, NodeState},
    cli::connect::{ConnectionCommandKind, ConnectionTarget, ConnectionView},
    config::{load_operator_config, resolve_secret_string, InlineOrPath, PgtmConfig, SecretSource},
    dcs::ClusterMemberView,
};
use reqwest::{Certificate, Identity, Method, StatusCode, Url};
use rustls::{
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
    ClientConfig, RootCertStore,
};
use serde::{Deserialize, Serialize};
use serde_json::error::Category as JsonErrorCategory;
use tokio::fs;
use tokio_postgres::{Client as PostgresClient, Config as PostgresConfig, Row};
use tokio_postgres_rustls::MakeRustlsConnect;

const REQUEST_POLL_DELAY: Duration = Duration::from_millis(100);
const API_TIMEOUT: Duration = Duration::from_secs(5);

pub const CONTAINER_SCENARIO_DIR: &str = "/var/lib/pgtuskmaster/ha-runner/scenario";
pub const CONTAINER_MATERIALIZED_DIR: &str = "/var/lib/pgtuskmaster/ha-runner/materialized";
pub const CONTAINER_CONTRACT_DIR: &str = "/var/lib/pgtuskmaster/ha-runner/contract";
pub const CONTAINER_ARTIFACTS_DIR: &str = "/var/lib/pgtuskmaster/ha-runner/artifacts";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerSeedSelection {
    Automatic,
    ViaMember { member_id: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunnerCommand {
    Ping,
    ClusterStatus {
        seed: RunnerSeedSelection,
    },
    PrimaryTls,
    WritablePrimaryTls,
    ReplicasTls,
    SwitchoverRequest {
        via_member_id: String,
        target_member_id: Option<String>,
    },
    ExecuteSql {
        dsn: String,
        sql: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerRequest {
    pub request_id: String,
    pub command: RunnerCommand,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunnerResponsePayload {
    Pong,
    State { state: Box<NodeState> },
    ConnectionView { view: ConnectionView },
    WritablePrimaryTarget { target: WritablePrimaryTarget },
    Accepted { accepted: AcceptedResponse },
    SqlRows { rows: Vec<String> },
    Text { value: String },
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WritablePrimaryTarget {
    pub authority_member_id: String,
    pub route: ConnectionTarget,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerResponse {
    pub request_id: String,
    pub payload: RunnerResponsePayload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerContractPaths {
    pub request_path: PathBuf,
    pub progress_path: PathBuf,
    pub result_path: PathBuf,
}

impl RunnerContractPaths {
    pub fn from_dir(contract_dir: &Path) -> Self {
        Self {
            request_path: contract_dir.join("launch-request.json"),
            progress_path: contract_dir.join("progress.jsonl"),
            result_path: contract_dir.join("result.json"),
        }
    }
}

pub async fn run_daemon(contract_dir: &Path) -> Result<(), String> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| "install rustls crypto provider for ha runner failed".to_string())?;
    let contract = RunnerContractPaths::from_dir(contract_dir);
    append_progress(
        contract.progress_path.as_path(),
        "runner_started",
        serde_json::json!({
            "contract_dir": contract_dir,
        }),
    )
    .await?;

    let mut sql_clients = BTreeMap::new();
    let mut last_request_id = String::new();
    loop {
        let request = match read_request(contract.request_path.as_path()).await? {
            Some(request) => request,
            None => {
                tokio::time::sleep(REQUEST_POLL_DELAY).await;
                continue;
            }
        };
        if request.request_id == last_request_id {
            tokio::time::sleep(REQUEST_POLL_DELAY).await;
            continue;
        }

        append_progress(
            contract.progress_path.as_path(),
            "request_received",
            serde_json::json!({
                "request_id": request.request_id,
                "command": request.command,
            }),
        )
        .await?;

        let response = match handle_request(&request, &mut sql_clients).await {
            Ok(payload) => RunnerResponse {
                request_id: request.request_id.clone(),
                payload,
            },
            Err(message) => RunnerResponse {
                request_id: request.request_id.clone(),
                payload: RunnerResponsePayload::Error { message },
            },
        };
        write_response(contract.result_path.as_path(), &response).await?;
        append_progress(
            contract.progress_path.as_path(),
            "request_finished",
            serde_json::json!({
                "request_id": response.request_id,
                "response_kind": response_kind_label(&response.payload),
            }),
        )
        .await?;
        last_request_id = response.request_id;
    }
}

async fn handle_request(
    request: &RunnerRequest,
    sql_clients: &mut BTreeMap<String, CachedSqlClient>,
) -> Result<RunnerResponsePayload, String> {
    match &request.command {
        RunnerCommand::Ping => Ok(RunnerResponsePayload::Pong),
        RunnerCommand::ClusterStatus { seed } => {
            select_seed(seed)
                .await
                .map(|(state, _)| RunnerResponsePayload::State {
                    state: Box::new(state),
                })
        }
        RunnerCommand::PrimaryTls => {
            let (state, config) = select_seed(&RunnerSeedSelection::Automatic).await?;
            build_primary_view(&state, &config)
                .map(|view| RunnerResponsePayload::ConnectionView { view })
        }
        RunnerCommand::WritablePrimaryTls => {
            let (state, config) = select_seed(&RunnerSeedSelection::Automatic).await?;
            build_writable_primary_target(&state, &config, sql_clients)
                .await
                .map(|target| RunnerResponsePayload::WritablePrimaryTarget { target })
        }
        RunnerCommand::ReplicasTls => {
            let (state, config) = select_seed(&RunnerSeedSelection::Automatic).await?;
            build_replicas_view(&state, &config)
                .map(|view| RunnerResponsePayload::ConnectionView { view })
        }
        RunnerCommand::SwitchoverRequest {
            via_member_id,
            target_member_id,
        } => {
            let config = load_seed_config(seed_config_path(via_member_id).as_path()).await?;
            request_switchover(&config, target_member_id.clone())
                .await
                .map(|accepted| RunnerResponsePayload::Accepted { accepted })
        }
        RunnerCommand::ExecuteSql { dsn, sql } => execute_sql(sql_clients, dsn, sql)
            .await
            .map(|rows| RunnerResponsePayload::SqlRows { rows }),
    }
}

async fn execute_sql(
    sql_clients: &mut BTreeMap<String, CachedSqlClient>,
    dsn: &str,
    sql: &str,
) -> Result<Vec<String>, String> {
    let client = match sql_clients.get_mut(dsn) {
        Some(client) => client,
        None => {
            let created = CachedSqlClient::connect(dsn).await?;
            let _ = sql_clients.insert(dsn.to_string(), created);
            sql_clients
                .get_mut(dsn)
                .ok_or_else(|| format!("sql client cache insert failed for dsn `{dsn}`"))?
        }
    };
    match client.execute(sql).await {
        Ok(rows) => Ok(rows),
        Err(SqlExecutionError::ConnectionClosed { message }) => {
            let replacement = CachedSqlClient::connect(dsn).await?;
            *client = replacement;
            client
                .execute(sql)
                .await
                .map_err(|err| err.into_message())
                .map_err(|follow_up| {
                    format!("{message}; reconnect retry also failed for dsn `{dsn}`: {follow_up}")
                })
        }
        Err(err) => Err(err.into_message()),
    }
}

async fn read_request(path: &Path) -> Result<Option<RunnerRequest>, String> {
    match fs::read_to_string(path).await {
        Ok(contents) => match serde_json::from_str(contents.as_str()) {
            Ok(request) => Ok(Some(request)),
            Err(err) if is_transient_json_read_error(&err) => Ok(None),
            Err(err) => Err(format!("parse runner request failed: {err}")),
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!(
            "read runner request `{}` failed: {err}",
            path.display()
        )),
    }
}

async fn write_response(path: &Path, response: &RunnerResponse) -> Result<(), String> {
    let rendered = serde_json::to_string_pretty(response)
        .map_err(|err| format!("serialize runner response failed: {err}"))?;
    fs::write(path, rendered)
        .await
        .map_err(|err| format!("write runner response `{}` failed: {err}", path.display()))
}

async fn append_progress(path: &Path, kind: &str, detail: serde_json::Value) -> Result<(), String> {
    let mut line = serde_json::json!({
        "kind": kind,
        "detail": detail,
    })
    .to_string();
    line.push('\n');
    use tokio::io::AsyncWriteExt as _;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|err| format!("open runner progress `{}` failed: {err}", path.display()))?;
    file.write_all(line.as_bytes())
        .await
        .map_err(|err| format!("append runner progress `{}` failed: {err}", path.display()))
}

fn response_kind_label(payload: &RunnerResponsePayload) -> &'static str {
    match payload {
        RunnerResponsePayload::Pong => "pong",
        RunnerResponsePayload::State { .. } => "state",
        RunnerResponsePayload::ConnectionView { .. } => "connection_view",
        RunnerResponsePayload::WritablePrimaryTarget { .. } => "writable_primary_target",
        RunnerResponsePayload::Accepted { .. } => "accepted",
        RunnerResponsePayload::SqlRows { .. } => "sql_rows",
        RunnerResponsePayload::Text { .. } => "text",
        RunnerResponsePayload::Error { .. } => "error",
    }
}

fn is_transient_json_read_error(err: &serde_json::Error) -> bool {
    err.classify() == JsonErrorCategory::Eof
}

#[derive(Clone)]
struct SeedConfig {
    operator: PgtmConfig,
}

async fn select_seed(selection: &RunnerSeedSelection) -> Result<(NodeState, SeedConfig), String> {
    match selection {
        RunnerSeedSelection::Automatic => {
            let mut best_state = None;
            let mut best_score = None;
            let mut errors = Vec::new();
            for member_id in runner_seed_member_ids() {
                let config_path = seed_config_path(member_id);
                match load_seed_config(config_path.as_path()).await {
                    Ok(config) => match fetch_state(&config).await {
                        Ok(state) => {
                            let score = status_score(&state);
                            match best_score {
                                Some(previous) if previous >= score => {}
                                _ => {
                                    best_score = Some(score);
                                    best_state = Some((state, config));
                                }
                            }
                        }
                        Err(err) => errors.push(format!("{member_id}: {err}")),
                    },
                    Err(err) => errors.push(format!("{member_id}: {err}")),
                }
            }
            best_state.ok_or_else(|| {
                format!(
                    "runner cluster status failed for every seed:\n{}",
                    errors.join("\n")
                )
            })
        }
        RunnerSeedSelection::ViaMember { member_id } => {
            let config = load_seed_config(seed_config_path(member_id).as_path()).await?;
            let state = fetch_state(&config).await?;
            Ok((state, config))
        }
    }
}

fn runner_seed_member_ids() -> [&'static str; 3] {
    ["node-a", "node-b", "node-c"]
}

fn seed_config_path(member_id: &str) -> PathBuf {
    PathBuf::from(format!(
        "/etc/pgtuskmaster/ha-runner/seeds/{member_id}.toml"
    ))
}

async fn load_seed_config(path: &Path) -> Result<SeedConfig, String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        load_operator_config(path.as_path())
            .map(|operator| SeedConfig { operator })
            .map_err(|err| format!("load runner seed config `{}` failed: {err}", path.display()))
    })
    .await
    .map_err(|err| format!("join load runner seed config task failed: {err}"))?
}

async fn fetch_state(config: &SeedConfig) -> Result<NodeState, String> {
    let api_client = build_raw_api_client(config)?;
    let path = join_api_path(config, "/state")?;
    let mut request = api_client.request(Method::GET, path);
    if let Some(token) = resolve_read_token(config)? {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .await
        .map_err(|err| format!("send runner state request failed: {err}"))?;
    if response.status() != StatusCode::OK {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| format!("read runner state error body failed: {err}"))?;
        return Err(format!(
            "runner state request returned unexpected status {status}: {body}"
        ));
    }
    response
        .json::<NodeState>()
        .await
        .map_err(|err| format!("decode runner state response failed: {err}"))
}

async fn request_switchover(
    config: &SeedConfig,
    target_member_id: Option<String>,
) -> Result<AcceptedResponse, String> {
    let api_client = build_raw_api_client(config)?;
    let path = join_api_path(config, "/switchover")?;
    let response = api_client
        .request(Method::POST, path)
        .bearer_auth(
            resolve_admin_token(config)?
                .ok_or_else(|| "runner switchover request requires an admin token".to_string())?,
        )
        .json(&serde_json::json!({
            "switchover_to": target_member_id,
        }))
        .send()
        .await
        .map_err(|err| format!("send switchover request failed: {err}"))?;
    if response.status() != StatusCode::ACCEPTED {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| format!("read switchover error body failed: {err}"))?;
        return Err(format!(
            "switchover request returned unexpected status {status}: {body}"
        ));
    }
    response
        .json::<AcceptedResponse>()
        .await
        .map_err(|err| format!("decode switchover response failed: {err}"))
}

fn build_primary_view(state: &NodeState, config: &SeedConfig) -> Result<ConnectionView, String> {
    let primary_id = authority_primary_member(state).ok_or_else(|| {
        "seed state does not currently expose an authoritative primary".to_string()
    })?;
    let member = state
        .dcs
        .cluster()
        .and_then(|cluster| cluster.member(&pgtuskmaster_rust::state::MemberId(primary_id.clone())))
        .ok_or_else(|| {
            format!("authoritative primary `{primary_id}` is not present in the DCS member slots")
        })?;
    let target = build_connection_target(member, config, config.operator.primary_target.as_ref())?;
    Ok(build_connection_view(
        state,
        ConnectionCommandKind::Primary,
        vec![target],
    ))
}

async fn build_writable_primary_target(
    state: &NodeState,
    config: &SeedConfig,
    sql_clients: &mut BTreeMap<String, CachedSqlClient>,
) -> Result<WritablePrimaryTarget, String> {
    let primary_id = authority_primary_member(state).ok_or_else(|| {
        "seed state does not currently expose an authoritative primary".to_string()
    })?;
    let member = state
        .dcs
        .cluster()
        .and_then(|cluster| cluster.member(&pgtuskmaster_rust::state::MemberId(primary_id.clone())))
        .ok_or_else(|| {
            format!("authoritative primary `{primary_id}` is not present in the DCS member slots")
        })?;
    let route = build_connection_target(member, config, config.operator.primary_target.as_ref())?;
    probe_writable_primary(sql_clients, route.dsn.as_str()).await?;
    Ok(WritablePrimaryTarget {
        authority_member_id: primary_id,
        route,
    })
}

fn build_replicas_view(state: &NodeState, config: &SeedConfig) -> Result<ConnectionView, String> {
    let targets = state
        .dcs
        .cluster()
        .into_iter()
        .flat_map(|cluster| {
            cluster
                .member_ids()
                .filter_map(|member_id| cluster.member(member_id))
        })
        .filter(|member| member_is_ready_replica(member))
        .map(|member| build_connection_target(member, config, None))
        .collect::<Result<Vec<_>, _>>()?;
    if targets.is_empty() {
        return Err("seed state does not currently expose any ready replica members".to_string());
    }
    Ok(build_connection_view(
        state,
        ConnectionCommandKind::Replicas,
        targets,
    ))
}

async fn probe_writable_primary(
    sql_clients: &mut BTreeMap<String, CachedSqlClient>,
    dsn: &str,
) -> Result<(), String> {
    let probe_sql =
        "CREATE TEMP TABLE pgtm_writable_primary_probe ON COMMIT DROP AS SELECT 'probe'::text AS token;";
    let _ = execute_sql(sql_clients, dsn, probe_sql).await?;
    Ok(())
}

fn build_connection_view(
    state: &NodeState,
    kind: ConnectionCommandKind,
    targets: Vec<ConnectionTarget>,
) -> ConnectionView {
    ConnectionView {
        cluster_name: state.cluster_name.clone(),
        scope: state.scope.clone(),
        kind,
        tls: true,
        discovered_member_count: state
            .dcs
            .cluster()
            .map(|cluster| cluster.member_count())
            .unwrap_or(0),
        warnings: Vec::new(),
        targets,
    }
}

fn build_connection_target(
    member: &ClusterMemberView,
    config: &SeedConfig,
    override_target: Option<&pgtuskmaster_rust::config::PgtmPrimaryTargetConfig>,
) -> Result<ConnectionTarget, String> {
    let resolved_host = override_target
        .map(|target| target.host.trim())
        .unwrap_or_else(|| member.postgres_target().host().trim());
    let resolved_port = override_target
        .and_then(|target| target.port)
        .unwrap_or(member.postgres_target().port());
    if resolved_host.is_empty() || resolved_port == 0 {
        return Err("member does not advertise PostgreSQL host/port".to_string());
    }
    let dsn = render_connection_dsn(resolved_host, resolved_port, config)?;
    Ok(ConnectionTarget {
        member_id: member.postgres_target().host().to_string(),
        postgres_host: resolved_host.to_string(),
        postgres_port: resolved_port,
        dsn,
    })
}

fn render_connection_dsn(host: &str, port: u16, config: &SeedConfig) -> Result<String, String> {
    let mut fields = vec![
        ("host", host.to_string()),
        ("port", port.to_string()),
        ("user", "postgres".to_string()),
        ("dbname", "postgres".to_string()),
        ("sslmode", "verify-full".to_string()),
    ];
    let ca_path = config
        .operator
        .postgres
        .tls
        .ca_cert
        .as_ref()
        .or(config.operator.api.tls.ca_cert.as_ref())
        .and_then(inline_or_path_to_path)
        .ok_or_else(|| "runner postgres TLS CA path is not path-backed".to_string())?;
    let identity = config
        .operator
        .postgres
        .tls
        .identity
        .as_ref()
        .or(config.operator.api.tls.identity.as_ref())
        .ok_or_else(|| "runner postgres TLS identity is missing".to_string())?;
    let cert_path = inline_or_path_to_path(&identity.cert)
        .ok_or_else(|| "runner postgres client certificate is not path-backed".to_string())?;
    let key_path = secret_to_path(&identity.key)
        .ok_or_else(|| "runner postgres client key is not path-backed".to_string())?;
    fields.push(("sslrootcert", ca_path.to_string_lossy().into_owned()));
    fields.push(("sslcert", cert_path.to_string_lossy().into_owned()));
    fields.push(("sslkey", key_path.to_string_lossy().into_owned()));
    Ok(fields
        .iter()
        .map(|(key, value)| format!("{key}={}", escape_libpq_value(value.as_str())))
        .collect::<Vec<_>>()
        .join(" "))
}

fn build_raw_api_client(config: &SeedConfig) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder().timeout(API_TIMEOUT);
    if let Some(source) = config.operator.api.tls.ca_cert.as_ref() {
        let pem = resolve_inline_or_path_bytes(source, "pgtm.api.tls.ca_cert")?;
        let certificate = Certificate::from_pem(pem.as_slice())
            .map_err(|err| format!("parse api CA failed: {err}"))?;
        builder = builder.add_root_certificate(certificate);
    }
    if let Some(identity) = config.operator.api.tls.identity.as_ref() {
        let cert = resolve_inline_or_path_bytes(&identity.cert, "pgtm.api.tls.identity.cert")?;
        let key = resolve_secret_bytes(&identity.key, "pgtm.api.tls.identity.key")?;
        let mut pem = Vec::with_capacity(cert.len().saturating_add(key.len()));
        pem.extend_from_slice(cert.as_slice());
        pem.extend_from_slice(key.as_slice());
        let identity = Identity::from_pem(pem.as_slice())
            .map_err(|err| format!("parse api client identity failed: {err}"))?;
        builder = builder.identity(identity);
    }
    builder
        .build()
        .map_err(|err| format!("build runner raw api client failed: {err}"))
}

fn join_api_path(config: &SeedConfig, path: &str) -> Result<reqwest::Url, String> {
    let base_url = config
        .operator
        .api
        .base_url
        .as_deref()
        .ok_or_else(|| "runner seed config is missing `pgtm.api.base_url`".to_string())?;
    let base_url = Url::parse(base_url).map_err(|err| format!("parse base url failed: {err}"))?;
    base_url
        .join(path)
        .map_err(|err| format!("join api path `{path}` failed: {err}"))
}

fn resolve_read_token(config: &SeedConfig) -> Result<Option<String>, String> {
    match &config.operator.api.auth {
        pgtuskmaster_rust::config::PgtmApiAuthConfig::Disabled => Ok(None),
        pgtuskmaster_rust::config::PgtmApiAuthConfig::RoleTokens {
            read_token,
            admin_token: _,
        } => {
            let read = match read_token {
                Some(source) => resolve_secret_string("pgtm.api.auth.read_token", source)
                    .map(Some)
                    .map_err(|err| err.to_string())?,
                None => None,
            };
            if read.is_some() {
                Ok(read)
            } else {
                resolve_admin_token(config)
            }
        }
    }
}

fn resolve_admin_token(config: &SeedConfig) -> Result<Option<String>, String> {
    match &config.operator.api.auth {
        pgtuskmaster_rust::config::PgtmApiAuthConfig::Disabled => Ok(None),
        pgtuskmaster_rust::config::PgtmApiAuthConfig::RoleTokens { admin_token, .. } => admin_token
            .as_ref()
            .map(|source| {
                resolve_secret_string("pgtm.api.auth.admin_token", source)
                    .map_err(|err| err.to_string())
            })
            .transpose(),
    }
}

fn inline_or_path_to_path(source: &InlineOrPath) -> Option<PathBuf> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => Some(path.clone()),
        InlineOrPath::Inline { .. } => None,
    }
}

fn secret_to_path(source: &SecretSource) -> Option<PathBuf> {
    match source {
        SecretSource::Path(path) | SecretSource::PathConfig { path } => Some(path.clone()),
        SecretSource::Inline { .. } | SecretSource::Env { .. } => None,
    }
}

fn resolve_inline_or_path_bytes(source: &InlineOrPath, label: &str) -> Result<Vec<u8>, String> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => std::fs::read(path)
            .map_err(|err| format!("read `{label}` at `{}` failed: {err}", path.display())),
        InlineOrPath::Inline { content } => Ok(content.as_bytes().to_vec()),
    }
}

fn resolve_secret_bytes(source: &SecretSource, label: &str) -> Result<Vec<u8>, String> {
    match source {
        SecretSource::Path(path) | SecretSource::PathConfig { path } => std::fs::read(path)
            .map_err(|err| format!("read `{label}` at `{}` failed: {err}", path.display())),
        SecretSource::Inline { content } => Ok(content.as_bytes().to_vec()),
        SecretSource::Env { env } => std::env::var(env)
            .map(|value| value.into_bytes())
            .map_err(|err| format!("read `{label}` from env `{env}` failed: {err}")),
    }
}

fn authority_primary_member(state: &NodeState) -> Option<String> {
    match &state.ha.publication {
        pgtuskmaster_rust::ha::types::PublicationState::Projected(
            pgtuskmaster_rust::ha::types::AuthorityProjection::Primary(epoch),
        ) => Some(epoch.holder.0.clone()),
        pgtuskmaster_rust::ha::types::PublicationState::Unknown
        | pgtuskmaster_rust::ha::types::PublicationState::Projected(
            pgtuskmaster_rust::ha::types::AuthorityProjection::NoPrimary(_),
        ) => None,
    }
}

fn member_is_ready_replica(member: &ClusterMemberView) -> bool {
    matches!(
        member.postgres(),
        pgtuskmaster_rust::dcs::MemberPostgresView::Replica {
            readiness: pgtuskmaster_rust::pginfo::state::Readiness::Ready,
            ..
        }
    )
}

fn status_score(status: &NodeState) -> (usize, usize, usize, usize) {
    let reported_primary_count = status
        .dcs
        .cluster()
        .into_iter()
        .flat_map(|cluster| cluster.members())
        .filter(|(_member_id, member)| {
            matches!(
                member.postgres(),
                pgtuskmaster_rust::dcs::MemberPostgresView::Primary { .. }
            )
        })
        .count();
    let discovered_member_count = status
        .dcs
        .cluster()
        .map(|cluster| cluster.member_count())
        .unwrap_or_default();
    (
        discovered_member_count,
        usize::from(status.dcs.mode() == pgtuskmaster_rust::dcs::DcsMode::Coordinated),
        usize::from(matches!(
            &status.ha.publication,
            pgtuskmaster_rust::ha::types::PublicationState::Projected(
                pgtuskmaster_rust::ha::types::AuthorityProjection::Primary(_)
            )
        )),
        usize::from(reported_primary_count == 1),
    )
}

fn escape_libpq_value(value: &str) -> String {
    let requires_quotes = value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_whitespace() || ch == '\'' || ch == '\\');
    if !requires_quotes {
        return value.to_string();
    }

    let escaped = value.chars().fold(String::new(), |mut acc, ch| {
        match ch {
            '\'' | '\\' => {
                acc.push('\\');
                acc.push(ch);
            }
            _ => acc.push(ch),
        }
        acc
    });
    format!("'{escaped}'")
}

struct CachedSqlClient {
    client: PostgresClient,
}

impl CachedSqlClient {
    async fn connect(dsn: &str) -> Result<Self, String> {
        let config = build_postgres_config(dsn)?;
        let tls = build_postgres_tls_connector(dsn)?;
        let (client, connection) = config
            .connect(tls)
            .await
            .map_err(|err| format!("connect SQL dsn failed: {err}"))?;
        tokio::spawn(async move {
            if let Err(err) = connection.await {
                eprintln!("ha runner postgres connection error: {err}");
            }
        });
        Ok(Self { client })
    }

    async fn execute(&mut self, sql: &str) -> Result<Vec<String>, SqlExecutionError> {
        let rows = self.client.query(sql, &[]).await.map_err(|err| {
            if err.is_closed() {
                SqlExecutionError::ConnectionClosed {
                    message: format_runner_sql_error(&err),
                }
            } else {
                SqlExecutionError::QueryFailed {
                    message: format_runner_sql_error(&err),
                }
            }
        })?;
        Ok(flatten_rows(rows))
    }
}

fn format_runner_sql_error(err: &tokio_postgres::Error) -> String {
    match err.as_db_error() {
        Some(db_error) => format!(
            "execute runner SQL failed: {} (severity={} sqlstate={})",
            db_error.message(),
            db_error.severity(),
            db_error.code().code()
        ),
        None => format!("execute runner SQL failed: {err}"),
    }
}

enum SqlExecutionError {
    ConnectionClosed { message: String },
    QueryFailed { message: String },
}

impl SqlExecutionError {
    fn into_message(self) -> String {
        match self {
            Self::ConnectionClosed { message } | Self::QueryFailed { message } => message,
        }
    }
}

fn build_postgres_config(dsn: &str) -> Result<PostgresConfig, String> {
    let fields = parse_libpq_dsn_fields(dsn);
    let host = required_dsn_field(&fields, "host")?;
    let port = required_dsn_field(&fields, "port")?
        .parse::<u16>()
        .map_err(|err| format!("parse SQL dsn port failed: {err}"))?;
    let user = required_dsn_field(&fields, "user")?;
    let dbname = required_dsn_field(&fields, "dbname")?;

    let mut config = PostgresConfig::new();
    let _ = config.host(host.as_str());
    let _ = config.port(port);
    let _ = config.user(user.as_str());
    let _ = config.dbname(dbname.as_str());
    Ok(config)
}

fn required_dsn_field(fields: &BTreeMap<String, String>, key: &str) -> Result<String, String> {
    fields
        .get(key)
        .cloned()
        .ok_or_else(|| format!("runner SQL requires `{key}` in the dsn"))
}

fn flatten_rows(rows: Vec<Row>) -> Vec<String> {
    rows.into_iter()
        .flat_map(|row| {
            let column_count = row.columns().len();
            (0..column_count)
                .filter_map(move |index| row.try_get::<usize, Option<String>>(index).ok().flatten())
        })
        .collect()
}

fn build_postgres_tls_connector(dsn: &str) -> Result<MakeRustlsConnect, String> {
    let fields = parse_libpq_dsn_fields(dsn);
    let ssl_root_cert = fields
        .get("sslrootcert")
        .ok_or_else(|| "runner SQL requires sslrootcert in the dsn".to_string())?;
    let client_cert_path = fields
        .get("sslcert")
        .ok_or_else(|| "runner SQL requires sslcert in the dsn".to_string())?;
    let client_key_path = fields
        .get("sslkey")
        .ok_or_else(|| "runner SQL requires sslkey in the dsn".to_string())?;
    let mut roots = RootCertStore::empty();
    let ca_bytes = std::fs::read(ssl_root_cert)
        .map_err(|err| format!("read SQL root certificate `{ssl_root_cert}` failed: {err}"))?;
    let certs = CertificateDer::pem_slice_iter(ca_bytes.as_slice())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("parse SQL root certificate PEM failed: {err}"))?;
    let _ = roots.add_parsable_certificates(certs);

    let client_cert_bytes = std::fs::read(client_cert_path)
        .map_err(|err| format!("read SQL client certificate `{client_cert_path}` failed: {err}"))?;
    let client_key_bytes = std::fs::read(client_key_path)
        .map_err(|err| format!("read SQL client key `{client_key_path}` failed: {err}"))?;
    let client_certs = CertificateDer::pem_slice_iter(client_cert_bytes.as_slice())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("parse SQL client certificate PEM failed: {err}"))?;
    let client_key = PrivateKeyDer::from(
        PrivatePkcs8KeyDer::from_pem_slice(client_key_bytes.as_slice())
            .map_err(|err| format!("parse SQL client key PEM failed: {err}"))?,
    );
    let tls = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_client_auth_cert(client_certs, client_key)
        .map_err(|err| format!("build SQL rustls client config failed: {err}"))?;
    Ok(MakeRustlsConnect::new(tls))
}

fn parse_libpq_dsn_fields(dsn: &str) -> BTreeMap<String, String> {
    dsn.split_whitespace()
        .filter_map(|field| {
            let (key, value) = field.split_once('=')?;
            Some((key.to_string(), unescape_libpq_value(value)))
        })
        .collect()
}

fn unescape_libpq_value(value: &str) -> String {
    if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
        let inner = &value[1..value.len().saturating_sub(1)];
        inner
            .chars()
            .fold((false, String::new()), |(escaped, mut acc), ch| {
                if escaped {
                    acc.push(ch);
                    (false, acc)
                } else if ch == '\\' {
                    (true, acc)
                } else {
                    acc.push(ch);
                    (false, acc)
                }
            })
            .1
    } else {
        value.to_string()
    }
}
