use std::{sync::Arc, time::Duration};

use rustls::ServerConfig;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{server::TlsStream, TlsAcceptor};

use crate::{
    api::{
        controller::{delete_switchover, get_ha_state, post_switchover, SwitchoverRequestInput},
        fallback::{get_fallback_cluster, post_fallback_heartbeat, FallbackHeartbeatInput},
        ApiError,
    },
    config::RuntimeConfig,
    dcs::store::DcsStore,
    debug_api::{snapshot::SystemSnapshot, view::build_verbose_payload},
    state::{StateSubscriber, WorkerError},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiTlsMode {
    Disabled,
    Optional,
    Required,
}

#[derive(Clone, Debug, Default)]
struct ApiRoleTokens {
    read_token: Option<String>,
    admin_token: Option<String>,
}

pub struct ApiWorkerCtx {
    listener: TcpListener,
    poll_interval: Duration,
    scope: String,
    config_subscriber: StateSubscriber<RuntimeConfig>,
    dcs_store: Box<dyn DcsStore>,
    debug_snapshot_subscriber: Option<StateSubscriber<SystemSnapshot>>,
    tls_mode_override: Option<ApiTlsMode>,
    tls_acceptor: Option<TlsAcceptor>,
    role_tokens: Option<ApiRoleTokens>,
    require_client_cert: bool,
}

impl ApiWorkerCtx {
    pub fn contract_stub(
        listener: TcpListener,
        config_subscriber: StateSubscriber<RuntimeConfig>,
        dcs_store: Box<dyn DcsStore>,
    ) -> Self {
        let scope = config_subscriber.latest().value.dcs.scope.clone();
        Self {
            listener,
            poll_interval: Duration::from_millis(10),
            scope,
            config_subscriber,
            dcs_store,
            debug_snapshot_subscriber: None,
            tls_mode_override: None,
            tls_acceptor: None,
            role_tokens: None,
            require_client_cert: false,
        }
    }

    pub fn local_addr(&self) -> Result<std::net::SocketAddr, WorkerError> {
        self.listener
            .local_addr()
            .map_err(|err| WorkerError::Message(format!("api local_addr failed: {err}")))
    }

    pub fn configure_tls(
        &mut self,
        mode: ApiTlsMode,
        server_config: Option<Arc<ServerConfig>>,
    ) -> Result<(), WorkerError> {
        match mode {
            ApiTlsMode::Disabled => {
                self.tls_mode_override = Some(ApiTlsMode::Disabled);
                self.tls_acceptor = None;
                Ok(())
            }
            ApiTlsMode::Optional | ApiTlsMode::Required => {
                let cfg = server_config.ok_or_else(|| {
                    WorkerError::Message(
                        "tls mode optional/required requires a server tls config".to_string(),
                    )
                })?;
                self.tls_mode_override = Some(mode);
                self.tls_acceptor = Some(TlsAcceptor::from(cfg));
                Ok(())
            }
        }
    }

    pub fn configure_role_tokens(
        &mut self,
        read_token: Option<String>,
        admin_token: Option<String>,
    ) -> Result<(), WorkerError> {
        let read = normalize_optional_token(read_token)?;
        let admin = normalize_optional_token(admin_token)?;

        if read.is_none() && admin.is_none() {
            self.role_tokens = None;
            return Ok(());
        }

        self.role_tokens = Some(ApiRoleTokens {
            read_token: read,
            admin_token: admin,
        });
        Ok(())
    }

    pub fn set_require_client_cert(&mut self, required: bool) {
        self.require_client_cert = required;
    }

    pub(crate) fn set_debug_snapshot_subscriber(
        &mut self,
        subscriber: StateSubscriber<SystemSnapshot>,
    ) {
        self.debug_snapshot_subscriber = Some(subscriber);
    }

    pub(crate) fn set_ha_snapshot_subscriber(
        &mut self,
        subscriber: StateSubscriber<SystemSnapshot>,
    ) {
        self.debug_snapshot_subscriber = Some(subscriber);
    }
}

pub async fn run(mut ctx: ApiWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub async fn step_once(ctx: &mut ApiWorkerCtx) -> Result<(), WorkerError> {
    let (stream, _peer) =
        match tokio::time::timeout(Duration::from_millis(1), ctx.listener.accept()).await {
            Ok(Ok((stream, peer))) => (stream, peer),
            Ok(Err(err)) => {
                return Err(WorkerError::Message(format!("api accept failed: {err}")));
            }
            Err(_elapsed) => return Ok(()),
        };

    let cfg = ctx.config_subscriber.latest().value;
    let mut stream = match accept_connection(ctx, &cfg, stream).await? {
        Some(stream) => stream,
        None => return Ok(()),
    };

    let request =
        match tokio::time::timeout(Duration::from_millis(100), stream.read_http_request()).await {
            Ok(Ok(req)) => req,
            Ok(Err(message)) => {
                let response = HttpResponse::text(400, "Bad Request", message);
                stream.write_http_response(response).await?;
                return Ok(());
            }
            Err(_elapsed) => return Ok(()),
        };

    match authorize_request(ctx, &cfg, &request) {
        AuthDecision::Allowed => {}
        AuthDecision::Unauthorized => {
            let response = HttpResponse::text(401, "Unauthorized", "unauthorized");
            stream.write_http_response(response).await?;
            return Ok(());
        }
        AuthDecision::Forbidden => {
            let response = HttpResponse::text(403, "Forbidden", "forbidden");
            stream.write_http_response(response).await?;
            return Ok(());
        }
    }

    let response = route_request(ctx, &cfg, request);
    stream.write_http_response(response).await?;
    Ok(())
}

fn route_request(
    ctx: &mut ApiWorkerCtx,
    cfg: &RuntimeConfig,
    request: HttpRequest,
) -> HttpResponse {
    let (path, query) = split_path_and_query(&request.path);
    match (request.method.as_str(), path) {
        ("POST", "/switchover") => {
            let input = match serde_json::from_slice::<SwitchoverRequestInput>(&request.body) {
                Ok(parsed) => parsed,
                Err(err) => {
                    return HttpResponse::text(400, "Bad Request", format!("invalid json: {err}"));
                }
            };
            match post_switchover(&ctx.scope, &mut *ctx.dcs_store, input) {
                Ok(value) => HttpResponse::json(202, "Accepted", &value),
                Err(err) => api_error_to_http(err),
            }
        }
        ("DELETE", "/ha/switchover") => match delete_switchover(&ctx.scope, &mut *ctx.dcs_store) {
            Ok(value) => HttpResponse::json(202, "Accepted", &value),
            Err(err) => api_error_to_http(err),
        },
        ("GET", "/ha/state") => {
            let Some(subscriber) = ctx.debug_snapshot_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "snapshot unavailable");
            };
            let snapshot = subscriber.latest();
            let response = get_ha_state(&snapshot);
            HttpResponse::json(200, "OK", &response)
        }
        ("GET", "/fallback/cluster") => {
            let view = get_fallback_cluster(cfg);
            HttpResponse::json(200, "OK", &view)
        }
        ("POST", "/fallback/heartbeat") => {
            let input = match serde_json::from_slice::<FallbackHeartbeatInput>(&request.body) {
                Ok(parsed) => parsed,
                Err(err) => {
                    return HttpResponse::text(400, "Bad Request", format!("invalid json: {err}"));
                }
            };
            match post_fallback_heartbeat(input) {
                Ok(value) => HttpResponse::json(202, "Accepted", &value),
                Err(err) => api_error_to_http(err),
            }
        }
        ("GET", "/debug/snapshot") => {
            if !cfg.debug.enabled {
                return HttpResponse::text(404, "Not Found", "not found");
            }
            let Some(subscriber) = ctx.debug_snapshot_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "snapshot unavailable");
            };
            let snapshot = subscriber.latest();
            HttpResponse::text(200, "OK", format!("{:#?}", snapshot))
        }
        ("GET", "/debug/verbose") => {
            if !cfg.debug.enabled {
                return HttpResponse::text(404, "Not Found", "not found");
            }
            let Some(subscriber) = ctx.debug_snapshot_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "snapshot unavailable");
            };
            let since_sequence = match parse_since_sequence(query) {
                Ok(value) => value,
                Err(message) => return HttpResponse::text(400, "Bad Request", message),
            };
            let snapshot = subscriber.latest();
            let payload = build_verbose_payload(&snapshot, since_sequence);
            HttpResponse::json(200, "OK", &payload)
        }
        ("GET", "/debug/ui") => {
            if !cfg.debug.enabled {
                return HttpResponse::text(404, "Not Found", "not found");
            }
            HttpResponse::html(200, "OK", debug_ui_html())
        }
        _ => HttpResponse::text(404, "Not Found", "not found"),
    }
}

fn api_error_to_http(err: ApiError) -> HttpResponse {
    match err {
        ApiError::BadRequest(message) => HttpResponse::text(400, "Bad Request", message),
        ApiError::Unauthorized => HttpResponse::text(401, "Unauthorized", "unauthorized"),
        ApiError::DcsStore(message) => HttpResponse::text(503, "Service Unavailable", message),
        ApiError::Internal(message) => HttpResponse::text(500, "Internal Server Error", message),
    }
}

fn split_path_and_query(path: &str) -> (&str, Option<&str>) {
    match path.split_once('?') {
        Some((head, tail)) => (head, Some(tail)),
        None => (path, None),
    }
}

fn parse_since_sequence(query: Option<&str>) -> Result<Option<u64>, String> {
    let Some(query) = query else {
        return Ok(None);
    };

    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key == "since" {
            let parsed = value
                .parse::<u64>()
                .map_err(|err| format!("invalid since query parameter: {err}"))?;
            return Ok(Some(parsed));
        }
    }
    Ok(None)
}

fn debug_ui_html() -> &'static str {
    r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>PGTuskMaster Debug UI</title>
  <style>
    :root {
      --bg: radial-gradient(circle at 10% 10%, #162132, #081019 55%, #06090f 100%);
      --panel: rgba(16, 26, 40, 0.92);
      --line: rgba(139, 190, 255, 0.22);
      --text: #d8e6ff;
      --muted: #89a3c4;
      --ok: #4bd18b;
      --warn: #f0bc5e;
      --err: #ff7070;
      --accent: #5ec3ff;
      --font: "JetBrains Mono", "Fira Mono", Menlo, monospace;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: var(--font);
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      padding: 14px;
    }
    .layout {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
      gap: 12px;
      max-width: 1300px;
      margin: 0 auto;
    }
    .panel {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 12px;
      padding: 12px;
      box-shadow: inset 0 1px 0 rgba(255,255,255,0.04);
    }
    .panel h2 {
      margin: 0 0 10px 0;
      font-size: 14px;
      letter-spacing: 0.04em;
      color: var(--accent);
      text-transform: uppercase;
    }
    .metrics { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
    .metric {
      border: 1px solid var(--line);
      border-radius: 9px;
      padding: 8px;
      background: rgba(0,0,0,0.2);
    }
    .metric .label { font-size: 11px; color: var(--muted); text-transform: uppercase; }
    .metric .value { margin-top: 6px; font-size: 16px; font-weight: 700; }
    .badge {
      display: inline-flex;
      align-items: center;
      padding: 2px 8px;
      border-radius: 999px;
      font-size: 11px;
      border: 1px solid var(--line);
      margin-left: 8px;
    }
    .badge.ok { color: var(--ok); border-color: color-mix(in oklab, var(--ok), black 40%); }
    .badge.warn { color: var(--warn); border-color: color-mix(in oklab, var(--warn), black 40%); }
    .badge.err { color: var(--err); border-color: color-mix(in oklab, var(--err), black 40%); }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 12px;
    }
    th, td {
      text-align: left;
      padding: 6px;
      border-bottom: 1px solid rgba(255,255,255,0.08);
      vertical-align: top;
      word-break: break-word;
    }
    th { color: var(--muted); }
    .timeline { max-height: 260px; overflow: auto; }
    .full { grid-column: 1 / -1; }
    @media (max-width: 760px) {
      body { padding: 8px; }
      .metrics { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
  <div class="layout">
    <section class="panel full" id="meta-panel">
      <h2>Runtime Meta <span id="meta-badge" class="badge warn">loading</span></h2>
      <div class="metrics">
        <div class="metric"><div class="label">Lifecycle</div><div class="value" id="m-lifecycle">-</div></div>
        <div class="metric"><div class="label">Sequence</div><div class="value" id="m-seq">-</div></div>
        <div class="metric"><div class="label">Generated (ms)</div><div class="value" id="m-ts">-</div></div>
      </div>
    </section>
    <section class="panel" id="config-panel"><h2>Config</h2><div id="config-body">-</div></section>
    <section class="panel" id="pginfo-panel"><h2>PgInfo</h2><div id="pginfo-body">-</div></section>
    <section class="panel" id="dcs-panel"><h2>DCS</h2><div id="dcs-body">-</div></section>
    <section class="panel" id="process-panel"><h2>Process</h2><div id="process-body">-</div></section>
    <section class="panel" id="ha-panel"><h2>HA</h2><div id="ha-body">-</div></section>
    <section class="panel full timeline" id="timeline-panel">
      <h2>Timeline</h2>
      <table>
        <thead><tr><th>Seq</th><th>At</th><th>Category</th><th>Message</th></tr></thead>
        <tbody id="timeline-body"></tbody>
      </table>
    </section>
    <section class="panel full timeline" id="changes-panel">
      <h2>Changes</h2>
      <table>
        <thead><tr><th>Seq</th><th>At</th><th>Domain</th><th>Versions</th><th>Summary</th></tr></thead>
        <tbody id="changes-body"></tbody>
      </table>
    </section>
  </div>
  <script>
    const state = { since: 0 };
    const byId = (id) => document.getElementById(id);
    const asText = (value) => (value === null || value === undefined ? "-" : String(value));
    const badge = (label, cls) => {
      const el = byId("meta-badge");
      el.textContent = label;
      el.className = `badge ${cls}`;
    };
    function renderKeyValue(id, entries) {
      byId(id).innerHTML = entries
        .map(([k, v]) => `<div><strong>${k}</strong>: ${asText(v)}</div>`)
        .join("");
    }
    function renderRows(id, rows, mapRow) {
      byId(id).innerHTML = rows.map(mapRow).join("");
    }
    function render(payload) {
      byId("m-lifecycle").textContent = asText(payload.meta.app_lifecycle);
      byId("m-seq").textContent = asText(payload.meta.sequence);
      byId("m-ts").textContent = asText(payload.meta.generated_at_ms);
      badge("connected", "ok");

      renderKeyValue("config-body", [
        ["member", payload.config.member_id],
        ["cluster", payload.config.cluster_name],
        ["scope", payload.config.scope],
        ["version", payload.config.version],
        ["debug", payload.config.debug_enabled],
        ["tls", payload.config.tls_enabled]
      ]);
      renderKeyValue("pginfo-body", [
        ["variant", payload.pginfo.variant],
        ["worker", payload.pginfo.worker],
        ["sql", payload.pginfo.sql],
        ["readiness", payload.pginfo.readiness],
        ["summary", payload.pginfo.summary]
      ]);
      renderKeyValue("dcs-body", [
        ["worker", payload.dcs.worker],
        ["trust", payload.dcs.trust],
        ["members", payload.dcs.member_count],
        ["leader", payload.dcs.leader],
        ["switchover", payload.dcs.has_switchover_request]
      ]);
      renderKeyValue("process-body", [
        ["worker", payload.process.worker],
        ["state", payload.process.state],
        ["running_job", payload.process.running_job_id],
        ["last_outcome", payload.process.last_outcome]
      ]);
      renderKeyValue("ha-body", [
        ["worker", payload.ha.worker],
        ["phase", payload.ha.phase],
        ["tick", payload.ha.tick],
        ["pending", payload.ha.pending_actions]
      ]);

      renderRows("timeline-body", payload.timeline, (row) =>
        `<tr><td>${row.sequence}</td><td>${row.at_ms}</td><td>${row.category}</td><td>${row.message}</td></tr>`
      );
      renderRows("changes-body", payload.changes, (row) =>
        `<tr><td>${row.sequence}</td><td>${row.at_ms}</td><td>${row.domain}</td><td>${asText(row.previous_version)} -> ${asText(row.current_version)}</td><td>${row.summary}</td></tr>`
      );

      if (typeof payload.meta.sequence === "number") {
        state.since = Math.max(state.since, payload.meta.sequence);
      }
    }
    async function tick() {
      try {
        const response = await fetch(`/debug/verbose?since=${state.since}`, { cache: "no-store" });
        if (!response.ok) {
          badge(`http-${response.status}`, "warn");
          return;
        }
        const payload = await response.json();
        render(payload);
      } catch (err) {
        badge("offline", "err");
        console.error("debug ui fetch failed", err);
      }
    }
    tick();
    setInterval(tick, 900);
  </script>
</body>
</html>"#
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndpointRole {
    Read,
    Admin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthDecision {
    Allowed,
    Unauthorized,
    Forbidden,
}

fn authorize_request(
    ctx: &ApiWorkerCtx,
    cfg: &RuntimeConfig,
    request: &HttpRequest,
) -> AuthDecision {
    let tokens = resolve_role_tokens(ctx, cfg);
    if tokens.read_token.is_none() && tokens.admin_token.is_none() {
        return AuthDecision::Allowed;
    }

    let Some(token) = extract_bearer_token(request) else {
        return AuthDecision::Unauthorized;
    };

    if let Some(expected_admin) = tokens.admin_token.as_deref() {
        if token == expected_admin {
            return AuthDecision::Allowed;
        }
    }

    match endpoint_role(request) {
        EndpointRole::Read => {
            if let Some(expected_read) = tokens.read_token.as_deref() {
                if token == expected_read {
                    return AuthDecision::Allowed;
                }
            }
            AuthDecision::Unauthorized
        }
        EndpointRole::Admin => {
            if let Some(expected_read) = tokens.read_token.as_deref() {
                if token == expected_read {
                    return AuthDecision::Forbidden;
                }
            }
            AuthDecision::Unauthorized
        }
    }
}

fn resolve_role_tokens(ctx: &ApiWorkerCtx, cfg: &RuntimeConfig) -> ApiRoleTokens {
    if let Some(configured) = ctx.role_tokens.as_ref() {
        return configured.clone();
    }

    let read_from_api = normalize_runtime_token(cfg.api.read_auth_token.clone());
    let admin_from_api = normalize_runtime_token(cfg.api.admin_auth_token.clone());
    if read_from_api.is_some() || admin_from_api.is_some() {
        return ApiRoleTokens {
            read_token: read_from_api,
            admin_token: admin_from_api,
        };
    }

    let legacy = cfg.security.auth_token.clone();
    ApiRoleTokens {
        read_token: legacy.clone(),
        admin_token: legacy,
    }
}

fn endpoint_role(request: &HttpRequest) -> EndpointRole {
    let (path, _query) = split_path_and_query(&request.path);
    match (request.method.as_str(), path) {
        ("POST", "/switchover")
        | ("POST", "/fallback/heartbeat")
        | ("DELETE", "/ha/switchover") => EndpointRole::Admin,
        _ => EndpointRole::Read,
    }
}

fn normalize_optional_token(raw: Option<String>) -> Result<Option<String>, WorkerError> {
    match raw {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(WorkerError::Message(
                    "role token must not be empty when configured".to_string(),
                ))
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        None => Ok(None),
    }
}

fn normalize_runtime_token(raw: Option<String>) -> Option<String> {
    match raw {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => None,
    }
}

enum ApiConnection {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
}

impl ApiConnection {
    async fn write_http_response(&mut self, response: HttpResponse) -> Result<(), WorkerError> {
        match self {
            Self::Plain(stream) => write_http_response(stream, response).await,
            Self::Tls(stream) => write_http_response(stream, response).await,
        }
    }

    async fn read_http_request(&mut self) -> Result<HttpRequest, String> {
        match self {
            Self::Plain(stream) => read_http_request(stream).await,
            Self::Tls(stream) => read_http_request(stream).await,
        }
    }
}

async fn accept_connection(
    ctx: &ApiWorkerCtx,
    cfg: &RuntimeConfig,
    stream: TcpStream,
) -> Result<Option<ApiConnection>, WorkerError> {
    match effective_tls_mode(ctx, cfg) {
        ApiTlsMode::Disabled => Ok(Some(ApiConnection::Plain(stream))),
        ApiTlsMode::Required => {
            let acceptor = require_tls_acceptor(ctx)?;
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    if ctx.require_client_cert && !has_peer_client_cert(&tls_stream) {
                        return Ok(None);
                    }
                    Ok(Some(ApiConnection::Tls(Box::new(tls_stream))))
                }
                Err(_) => Ok(None),
            }
        }
        ApiTlsMode::Optional => {
            if !looks_like_tls_client_hello(&stream).await? {
                return Ok(Some(ApiConnection::Plain(stream)));
            }

            let acceptor = require_tls_acceptor(ctx)?;
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    if ctx.require_client_cert && !has_peer_client_cert(&tls_stream) {
                        return Ok(None);
                    }
                    Ok(Some(ApiConnection::Tls(Box::new(tls_stream))))
                }
                Err(_) => Ok(None),
            }
        }
    }
}

fn effective_tls_mode(ctx: &ApiWorkerCtx, cfg: &RuntimeConfig) -> ApiTlsMode {
    if let Some(mode) = ctx.tls_mode_override {
        return mode;
    }

    if cfg.security.tls_enabled {
        ApiTlsMode::Required
    } else {
        ApiTlsMode::Disabled
    }
}

fn require_tls_acceptor(ctx: &ApiWorkerCtx) -> Result<TlsAcceptor, WorkerError> {
    ctx.tls_acceptor.clone().ok_or_else(|| {
        WorkerError::Message("tls mode requires a configured tls acceptor".to_string())
    })
}

fn has_peer_client_cert(stream: &TlsStream<TcpStream>) -> bool {
    let (_, connection) = stream.get_ref();
    connection
        .peer_certificates()
        .map(|certs| !certs.is_empty())
        .unwrap_or(false)
}

async fn looks_like_tls_client_hello(stream: &TcpStream) -> Result<bool, WorkerError> {
    let mut first = [0_u8; 1];
    match tokio::time::timeout(Duration::from_millis(10), stream.peek(&mut first)).await {
        Err(_) => Ok(false),
        Ok(Ok(0)) => Ok(false),
        Ok(Ok(_)) => Ok(first[0] == 0x16),
        Ok(Err(err)) if err.kind() == std::io::ErrorKind::WouldBlock => Ok(false),
        Ok(Err(err)) => Err(WorkerError::Message(format!("api tls peek failed: {err}"))),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpResponse {
    status: u16,
    reason: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
}

impl HttpResponse {
    fn text(status: u16, reason: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            reason,
            content_type: "text/plain; charset=utf-8",
            body: body.into().into_bytes(),
        }
    }

    fn json<T: serde::Serialize>(status: u16, reason: &'static str, value: &T) -> Self {
        match serde_json::to_vec(value) {
            Ok(body) => Self {
                status,
                reason,
                content_type: "application/json",
                body,
            },
            Err(err) => Self::text(
                500,
                "Internal Server Error",
                format!("json encode failed: {err}"),
            ),
        }
    }

    fn html(status: u16, reason: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            reason,
            content_type: "text/html; charset=utf-8",
            body: body.into().into_bytes(),
        }
    }
}

async fn write_http_response<S>(stream: &mut S, response: HttpResponse) -> Result<(), WorkerError>
where
    S: AsyncWrite + Unpin,
{
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        response.reason,
        response.content_type,
        response.body.len()
    );
    stream
        .write_all(header.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("api write header failed: {err}")))?;
    stream
        .write_all(&response.body)
        .await
        .map_err(|err| WorkerError::Message(format!("api write body failed: {err}")))?;
    Ok(())
}

async fn read_http_request<S>(stream: &mut S) -> Result<HttpRequest, String>
where
    S: AsyncRead + Unpin,
{
    const MAX_BYTES: usize = 1024 * 1024;
    const HEADER_LIMIT: usize = 16 * 1024;

    let mut buffer = Vec::<u8>::new();
    let mut temp = [0u8; 4096];
    let mut header_end: Option<usize> = None;
    let mut content_length: Option<usize> = None;

    loop {
        if buffer.len() > MAX_BYTES {
            return Err("request too large".to_string());
        }

        let n = stream
            .read(&mut temp)
            .await
            .map_err(|err| err.to_string())?;
        if n == 0 {
            return Err("client closed connection".to_string());
        }
        buffer.extend_from_slice(&temp[..n]);

        if header_end.is_none() {
            if let Some(pos) = find_header_end(&buffer) {
                header_end = Some(pos);
            } else if buffer.len() > HEADER_LIMIT {
                return Err("headers too large".to_string());
            }
        }

        if let Some(end) = header_end {
            if content_length.is_none() {
                content_length = parse_content_length(&buffer).map_err(|err| err.to_string())?;
            }
            let body_len = content_length.unwrap_or(0);
            let required = end.saturating_add(body_len);
            if buffer.len() >= required {
                break;
            }
        }
    }

    let mut headers = [httparse::Header {
        name: "",
        value: &[],
    }; 64];
    let mut req = httparse::Request::new(&mut headers);
    let status = req.parse(&buffer).map_err(|err| err.to_string())?;
    let header_bytes = match status {
        httparse::Status::Complete(bytes) => bytes,
        httparse::Status::Partial => return Err("incomplete http request".to_string()),
    };

    let method = req
        .method
        .ok_or_else(|| "missing http method".to_string())?
        .to_string();
    let path = req
        .path
        .ok_or_else(|| "missing http path".to_string())?
        .to_string();

    let mut parsed_headers = Vec::new();
    for header in req.headers.iter() {
        parsed_headers.push((
            header.name.to_string(),
            String::from_utf8_lossy(header.value).to_string(),
        ));
    }

    let body_len = content_length.unwrap_or(0);
    let body_end = header_bytes
        .checked_add(body_len)
        .ok_or_else(|| "content-length overflow".to_string())?;
    if body_end > buffer.len() {
        return Err("incomplete http body".to_string());
    }

    Ok(HttpRequest {
        method,
        path,
        headers: parsed_headers,
        body: buffer[header_bytes..body_end].to_vec(),
    })
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

fn parse_content_length(buffer: &[u8]) -> Result<Option<usize>, String> {
    let mut headers = [httparse::Header {
        name: "",
        value: &[],
    }; 64];
    let mut req = httparse::Request::new(&mut headers);
    let status = req.parse(buffer).map_err(|err| err.to_string())?;
    let _ = match status {
        httparse::Status::Complete(bytes) => bytes,
        httparse::Status::Partial => return Ok(None),
    };

    for header in req.headers.iter() {
        if header.name.eq_ignore_ascii_case("Content-Length") {
            let raw = String::from_utf8_lossy(header.value);
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(Some(0));
            }
            let parsed = trimmed
                .parse::<usize>()
                .map_err(|err| format!("invalid content-length: {err}"))?;
            return Ok(Some(parsed));
        }
    }
    Ok(Some(0))
}

fn extract_bearer_token(request: &HttpRequest) -> Option<String> {
    let header = request
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("Authorization"))
        .map(|(_, value)| value.as_str())?;

    let trimmed = header.trim();
    let prefix = "Bearer ";
    if !trimmed.starts_with(prefix) {
        return None;
    }
    Some(trimmed[prefix.len()..].trim().to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::{Arc, Mutex};

    use rcgen::{
        date_time_ymd, BasicConstraints, CertificateParams, DistinguishedName, DnType,
        ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair, KeyUsagePurpose,
    };
    use rustls::{
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName},
        ClientConfig, RootCertStore, ServerConfig,
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio_rustls::TlsConnector;

    use crate::{
        api::worker::{step_once, ApiTlsMode, ApiWorkerCtx},
        config::{
            ApiConfig, BinaryPaths, ClusterConfig, DcsConfig, DebugConfig, HaConfig,
            PostgresConfig, ProcessConfig, RuntimeConfig, SecurityConfig,
        },
        dcs::state::{DcsCache, DcsState, DcsTrust},
        dcs::store::{DcsStore, DcsStoreError, WatchEvent},
        debug_api::snapshot::{
            AppLifecycle, DebugChangeEvent, DebugDomain, DebugTimelineEntry, SystemSnapshot,
        },
        ha::{
            actions::HaAction,
            state::{HaPhase, HaState},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, UnixMillis, WorkerError},
        test_harness::{
            auth::ApiRoleTokens,
            namespace::NamespaceGuard,
            tls::{write_tls_material, TlsMode},
        },
    };

    #[derive(Clone, Default)]
    struct RecordingStore {
        writes: Arc<Mutex<Vec<(String, String)>>>,
        deletes: Arc<Mutex<Vec<String>>>,
    }

    impl RecordingStore {
        fn write_count(&self) -> Result<usize, WorkerError> {
            let guard = self
                .writes
                .lock()
                .map_err(|_| WorkerError::Message("writes lock poisoned".to_string()))?;
            Ok(guard.len())
        }

        fn writes(&self) -> Result<Vec<(String, String)>, WorkerError> {
            let guard = self
                .writes
                .lock()
                .map_err(|_| WorkerError::Message("writes lock poisoned".to_string()))?;
            Ok(guard.clone())
        }

        fn delete_count(&self) -> Result<usize, WorkerError> {
            let guard = self
                .deletes
                .lock()
                .map_err(|_| WorkerError::Message("deletes lock poisoned".to_string()))?;
            Ok(guard.len())
        }

        fn deletes(&self) -> Result<Vec<String>, WorkerError> {
            let guard = self
                .deletes
                .lock()
                .map_err(|_| WorkerError::Message("deletes lock poisoned".to_string()))?;
            Ok(guard.clone())
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            true
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(())
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
            let mut guard = self
                .deletes
                .lock()
                .map_err(|_| DcsStoreError::Io("deletes lock poisoned".to_string()))?;
            guard.push(path.to_string());
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(Vec::new())
        }
    }

    #[derive(Clone)]
    struct GeneratedCert {
        cert_der: Vec<u8>,
        key_der: Vec<u8>,
        cert_pem: String,
        key_pem: String,
    }

    impl GeneratedCert {
        fn cert_der(&self) -> CertificateDer<'static> {
            CertificateDer::from(self.cert_der.clone())
        }

        fn key_der(&self) -> PrivateKeyDer<'static> {
            PrivateKeyDer::from(PrivatePkcs8KeyDer::from(self.key_der.clone()))
        }
    }

    struct GeneratedCa {
        cert: GeneratedCert,
        issuer: Issuer<'static, KeyPair>,
    }

    fn sample_runtime_config(auth_token: Option<String>) -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: "/tmp/pgdata".into(),
                connect_timeout_s: 5,
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
            },
            ha: HaConfig {
                loop_interval_ms: 1000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 1000,
                bootstrap_timeout_ms: 1000,
                fencing_timeout_ms: 1000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    psql: "/usr/bin/psql".into(),
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:0".to_string(),
                read_auth_token: None,
                admin_auth_token: None,
            },
            debug: DebugConfig { enabled: true },
            security: SecurityConfig {
                tls_enabled: false,
                auth_token,
            },
        }
    }

    fn sample_pg_state() -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: crate::state::WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                pg_config: PgConfig {
                    port: Some(5432),
                    hot_standby: Some(false),
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
        }
    }

    fn sample_dcs_state(cfg: RuntimeConfig) -> DcsState {
        DcsState {
            worker: crate::state::WorkerStatus::Running,
            trust: DcsTrust::FullQuorum,
            cache: DcsCache {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config: cfg,
                init_lock: None,
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    fn sample_process_state() -> ProcessState {
        ProcessState::Idle {
            worker: crate::state::WorkerStatus::Running,
            last_outcome: None,
        }
    }

    fn sample_ha_state() -> HaState {
        HaState {
            worker: crate::state::WorkerStatus::Running,
            phase: HaPhase::Replica,
            tick: 7,
            pending: vec![HaAction::SignalFailSafe],
            recent_action_ids: BTreeSet::new(),
        }
    }

    fn sample_debug_snapshot(auth_token: Option<String>) -> SystemSnapshot {
        let cfg = sample_runtime_config(auth_token);
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        SystemSnapshot {
            app: AppLifecycle::Running,
            config: cfg_subscriber.latest(),
            pg: pg_subscriber.latest(),
            dcs: dcs_subscriber.latest(),
            process: process_subscriber.latest(),
            ha: ha_subscriber.latest(),
            generated_at: UnixMillis(1),
            sequence: 2,
            changes: vec![DebugChangeEvent {
                sequence: 1,
                at: UnixMillis(1),
                domain: DebugDomain::Config,
                previous_version: None,
                current_version: Some(cfg_subscriber.latest().version),
                summary: "config initialized".to_string(),
            }],
            timeline: vec![DebugTimelineEntry {
                sequence: 2,
                at: UnixMillis(1),
                domain: DebugDomain::Ha,
                message: "ha reached replica".to_string(),
            }],
        }
    }

    async fn build_ctx_with_config(
        cfg: RuntimeConfig,
    ) -> Result<(ApiWorkerCtx, RecordingStore), WorkerError> {
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

        let store = RecordingStore::default();
        let ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store.clone()));
        Ok((ctx, store))
    }

    async fn build_ctx(
        auth_token: Option<String>,
    ) -> Result<(ApiWorkerCtx, RecordingStore), WorkerError> {
        build_ctx_with_config(sample_runtime_config(auth_token)).await
    }

    fn extract_status_and_body(raw: &[u8]) -> Result<(String, Vec<u8>), WorkerError> {
        let raw_str = String::from_utf8_lossy(raw);
        let mut parts = raw_str.splitn(2, "\r\n");
        let status_line = parts
            .next()
            .ok_or_else(|| WorkerError::Message("missing status line".to_string()))?
            .to_string();
        let split = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .ok_or_else(|| WorkerError::Message("missing header terminator".to_string()))?;
        Ok((status_line, raw[split + 4..].to_vec()))
    }

    async fn send_plain_request(
        ctx: &mut ApiWorkerCtx,
        request_head: String,
        body: Option<Vec<u8>>,
    ) -> Result<(String, Vec<u8>), WorkerError> {
        let addr = ctx.local_addr()?;
        let mut client = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;

        client
            .write_all(request_head.as_bytes())
            .await
            .map_err(|err| WorkerError::Message(format!("client write header failed: {err}")))?;

        if let Some(body) = body {
            client
                .write_all(&body)
                .await
                .map_err(|err| WorkerError::Message(format!("client write body failed: {err}")))?;
        }

        step_once(ctx).await?;

        let mut raw = Vec::new();
        client
            .read_to_end(&mut raw)
            .await
            .map_err(|err| WorkerError::Message(format!("client read failed: {err}")))?;

        extract_status_and_body(&raw)
    }

    async fn send_tls_request(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
        request_head: String,
        body: Option<Vec<u8>>,
    ) -> Result<(String, Vec<u8>), WorkerError> {
        let addr = ctx.local_addr()?;
        let tcp = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;

        let connector = TlsConnector::from(client_config);
        let server_name = ServerName::try_from(server_name.to_string()).map_err(|err| {
            WorkerError::Message(format!("invalid server name {server_name}: {err}"))
        })?;

        let client = async move {
            let mut tls = connector
                .connect(server_name, tcp)
                .await
                .map_err(|err| WorkerError::Message(format!("tls connect failed: {err}")))?;
            tls.write_all(request_head.as_bytes())
                .await
                .map_err(|err| WorkerError::Message(format!("tls write header failed: {err}")))?;
            if let Some(body) = body {
                tls.write_all(&body)
                    .await
                    .map_err(|err| WorkerError::Message(format!("tls write body failed: {err}")))?;
            }
            let mut raw = Vec::new();
            if let Err(err) = tls.read_to_end(&mut raw).await {
                if err.kind() != std::io::ErrorKind::UnexpectedEof || raw.is_empty() {
                    return Err(WorkerError::Message(format!("tls read failed: {err}")));
                }
            }
            extract_status_and_body(&raw)
        };

        let (step_result, client_result) = tokio::join!(step_once(ctx), client);
        step_result?;
        client_result
    }

    async fn expect_tls_handshake_failure(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
    ) -> Result<(), WorkerError> {
        let addr = ctx.local_addr()?;
        let tcp = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;

        let connector = TlsConnector::from(client_config);
        let server_name = ServerName::try_from(server_name.to_string()).map_err(|err| {
            WorkerError::Message(format!("invalid server name {server_name}: {err}"))
        })?;

        let client = async move { connector.connect(server_name, tcp).await };
        let (step_result, client_result) = tokio::join!(step_once(ctx), client);
        step_result?;
        if client_result.is_ok() {
            return Err(WorkerError::Message(
                "expected tls handshake failure, but handshake succeeded".to_string(),
            ));
        }
        Ok(())
    }

    async fn expect_tls_request_rejected(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
    ) -> Result<(), WorkerError> {
        let result = send_tls_request(
            ctx,
            client_config,
            server_name,
            format_get("/fallback/cluster", None),
            None,
        )
        .await;

        match result {
            Ok((status, _body)) => {
                if status.contains("200") {
                    Err(WorkerError::Message(format!(
                        "expected tls request rejection, got successful status: {status}"
                    )))
                } else {
                    Ok(())
                }
            }
            Err(_) => Ok(()),
        }
    }

    fn generate_ca(common_name: &str) -> Result<GeneratedCa, WorkerError> {
        let mut params = CertificateParams::new(Vec::new())
            .map_err(|err| WorkerError::Message(format!("create ca params failed: {err}")))?;
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, common_name.to_string());
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages.push(KeyUsagePurpose::DigitalSignature);
        params.key_usages.push(KeyUsagePurpose::KeyCertSign);
        params.key_usages.push(KeyUsagePurpose::CrlSign);
        params.not_before = date_time_ymd(2024, 1, 1);
        params.not_after = date_time_ymd(2034, 1, 1);

        let key_pair = KeyPair::generate()
            .map_err(|err| WorkerError::Message(format!("generate ca key failed: {err}")))?;
        let cert = params
            .self_signed(&key_pair)
            .map_err(|err| WorkerError::Message(format!("self-sign ca failed: {err}")))?;

        Ok(GeneratedCa {
            cert: GeneratedCert {
                cert_der: cert.der().to_vec(),
                key_der: key_pair.serialize_der(),
                cert_pem: cert.pem(),
                key_pem: key_pair.serialize_pem(),
            },
            issuer: Issuer::new(params, key_pair),
        })
    }

    fn generate_leaf_cert(
        common_name: &str,
        dns_name: &str,
        expired: bool,
        issuer: &Issuer<'static, KeyPair>,
        is_client_cert: bool,
    ) -> Result<GeneratedCert, WorkerError> {
        let mut params = CertificateParams::new(vec![dns_name.to_string()])
            .map_err(|err| WorkerError::Message(format!("create leaf params failed: {err}")))?;
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, common_name.to_string());
        params.distinguished_name = dn;
        params.is_ca = IsCa::NoCa;
        params.key_usages.push(KeyUsagePurpose::DigitalSignature);
        if is_client_cert {
            params
                .extended_key_usages
                .push(ExtendedKeyUsagePurpose::ClientAuth);
        } else {
            params
                .extended_key_usages
                .push(ExtendedKeyUsagePurpose::ServerAuth);
        }
        if expired {
            params.not_before = date_time_ymd(2018, 1, 1);
            params.not_after = date_time_ymd(2019, 1, 1);
        } else {
            params.not_before = date_time_ymd(2024, 1, 1);
            params.not_after = date_time_ymd(2034, 1, 1);
        }

        let key_pair = KeyPair::generate()
            .map_err(|err| WorkerError::Message(format!("generate leaf key failed: {err}")))?;
        let cert = params
            .signed_by(&key_pair, issuer)
            .map_err(|err| WorkerError::Message(format!("sign leaf cert failed: {err}")))?;

        Ok(GeneratedCert {
            cert_der: cert.der().to_vec(),
            key_der: key_pair.serialize_der(),
            cert_pem: cert.pem(),
            key_pem: key_pair.serialize_pem(),
        })
    }

    fn build_server_config(
        server: &GeneratedCert,
        server_ca: &GeneratedCert,
    ) -> Result<Arc<ServerConfig>, WorkerError> {
        ensure_crypto_provider_installed()?;
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![server.cert_der(), server_ca.cert_der()],
                server.key_der(),
            )
            .map_err(|err| WorkerError::Message(format!("build server config failed: {err}")))?;
        Ok(Arc::new(config))
    }

    fn build_server_config_with_client_auth(
        server: &GeneratedCert,
        server_ca: &GeneratedCert,
        trusted_client_ca: &GeneratedCert,
    ) -> Result<Arc<ServerConfig>, WorkerError> {
        ensure_crypto_provider_installed()?;
        let mut roots = RootCertStore::empty();
        roots.add(trusted_client_ca.cert_der()).map_err(|err| {
            WorkerError::Message(format!("add trusted client root failed: {err}"))
        })?;

        let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(roots))
            .build()
            .map_err(|err| {
                WorkerError::Message(format!("build client cert verifier failed: {err}"))
            })?;

        let config = ServerConfig::builder()
            .with_client_cert_verifier(verifier)
            .with_single_cert(
                vec![server.cert_der(), server_ca.cert_der()],
                server.key_der(),
            )
            .map_err(|err| {
                WorkerError::Message(format!("build mTLS server config failed: {err}"))
            })?;

        Ok(Arc::new(config))
    }

    fn build_client_config(
        trusted_server_ca: &GeneratedCert,
        identity: Option<&GeneratedCert>,
        identity_ca: Option<&GeneratedCert>,
    ) -> Result<Arc<ClientConfig>, WorkerError> {
        ensure_crypto_provider_installed()?;
        let mut roots = RootCertStore::empty();
        roots.add(trusted_server_ca.cert_der()).map_err(|err| {
            WorkerError::Message(format!("add trusted server root failed: {err}"))
        })?;

        let builder = ClientConfig::builder().with_root_certificates(roots);
        let config = match identity {
            Some(cert) => builder
                .with_client_auth_cert(
                    vec![
                        cert.cert_der(),
                        identity_ca.map(GeneratedCert::cert_der).ok_or_else(|| {
                            WorkerError::Message(
                                "identity_ca is required when identity is configured".to_string(),
                            )
                        })?,
                    ],
                    cert.key_der(),
                )
                .map_err(|err| {
                    WorkerError::Message(format!("build mTLS client config failed: {err}"))
                })?,
            None => builder.with_no_client_auth(),
        };

        Ok(Arc::new(config))
    }

    fn ensure_crypto_provider_installed() -> Result<(), WorkerError> {
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }

        if rustls::crypto::CryptoProvider::get_default().is_some() {
            Ok(())
        } else {
            Err(WorkerError::Message(
                "rustls crypto provider is unavailable".to_string(),
            ))
        }
    }

    fn format_get(path: &str, auth: Option<&str>) -> String {
        match auth {
            Some(auth_header) => format!(
                "GET {path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: {auth_header}\r\n\r\n"
            ),
            None => format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n"),
        }
    }

    fn format_post(path: &str, auth: Option<&str>, body: &[u8]) -> String {
        match auth {
            Some(auth_header) => format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: {auth_header}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            ),
            None => format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            ),
        }
    }

    fn format_delete(path: &str, auth: Option<&str>) -> String {
        match auth {
            Some(auth_header) => format!(
                "DELETE {path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: {auth_header}\r\n\r\n"
            ),
            None => format!("DELETE {path} HTTP/1.1\r\nHost: localhost\r\n\r\n"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_role_permissions_allow_read_deny_admin() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-role-read-deny")?;

        let (mut ctx, store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let (status, _body) = send_plain_request(
            &mut ctx,
            format_get("/fallback/cluster", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");

        let post_body = br#"{"requested_by":"node-a"}"#.to_vec();
        let (status, _body) = send_plain_request(
            &mut ctx,
            format_post(
                "/switchover",
                Some(&roles.read_bearer_header()),
                post_body.as_slice(),
            ),
            Some(post_body),
        )
        .await?;
        assert!(status.contains("403"), "expected 403, got: {status}");
        assert_eq!(store.write_count()?, 0);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_role_permissions_allow_admin() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-role-admin-allow")?;

        let (mut ctx, store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let post_body = br#"{"requested_by":"node-a"}"#.to_vec();
        let (status, _body) = send_plain_request(
            &mut ctx,
            format_post(
                "/switchover",
                Some(&roles.admin_bearer_header()),
                post_body.as_slice(),
            ),
            Some(post_body),
        )
        .await?;
        assert!(status.contains("202"), "expected 202, got: {status}");
        assert_eq!(store.write_count()?, 1);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_state_route_returns_typed_json_even_when_debug_disabled() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-state-json")?;
        let mut cfg = sample_runtime_config(None);
        cfg.debug.enabled = false;
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let (status, body) =
            send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert!(status.contains("200"), "expected 200, got: {status}");
        let decoded: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|err| WorkerError::Message(format!("decode ha state json failed: {err}")))?;
        assert_eq!(decoded["cluster_name"], "cluster-a");
        assert_eq!(decoded["scope"], "scope-a");
        assert_eq!(decoded["self_member_id"], "node-a");
        assert_eq!(decoded["leader"], serde_json::Value::Null);
        assert_eq!(decoded["switchover_requested_by"], serde_json::Value::Null);
        assert_eq!(decoded["member_count"], 0);
        assert_eq!(decoded["dcs_trust"], "FullQuorum");
        assert_eq!(decoded["ha_phase"], "Replica");
        assert_eq!(decoded["ha_tick"], 7);
        assert_eq!(decoded["pending_actions"], 1);
        assert_eq!(decoded["snapshot_sequence"], 2);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_state_route_returns_503_without_subscriber() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-state-missing-subscriber")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let (status, _body) =
            send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert!(status.contains("503"), "expected 503, got: {status}");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_leader_routes_are_not_found_and_do_not_mutate_dcs_keys() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-leader-routes-removed")?;
        let (mut ctx, store) = build_ctx(None).await?;

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let (status, _body) = send_plain_request(
            &mut ctx,
            format_post("/ha/leader", None, body.as_slice()),
            Some(body),
        )
        .await?;
        assert!(status.contains("404"), "expected 404, got: {status}");

        let (status, _body) =
            send_plain_request(&mut ctx, format_delete("/ha/leader", None), None).await?;
        assert!(status.contains("404"), "expected 404, got: {status}");

        let (status, _body) =
            send_plain_request(&mut ctx, format_delete("/ha/switchover", None), None).await?;
        assert!(status.contains("202"), "expected 202, got: {status}");

        assert_eq!(store.write_count()?, 0);

        assert_eq!(store.delete_count()?, 1);
        let deletes = store.deletes()?;
        assert_eq!(deletes, vec!["/scope-a/switchover"]);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_role_permissions_handle_removed_ha_leader_routes() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-authz-removed-leader-routes")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let (status, _body) = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let (status, _body) = send_plain_request(
            &mut ctx,
            format_post(
                "/ha/leader",
                Some(&roles.read_bearer_header()),
                body.as_slice(),
            ),
            Some(body),
        )
        .await?;
        assert!(status.contains("404"), "expected 404, got: {status}");

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let (status, _body) = send_plain_request(
            &mut ctx,
            format_post(
                "/ha/leader",
                Some(&roles.admin_bearer_header()),
                body.as_slice(),
            ),
            Some(body),
        )
        .await?;
        assert!(status.contains("404"), "expected 404, got: {status}");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_legacy_auth_token_fallback_protects_ha_routes() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-authz-legacy-fallback")?;
        let (mut ctx, _store) = build_ctx(Some("legacy-token".to_string())).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let (status, _body) =
            send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert!(status.contains("401"), "expected 401, got: {status}");

        let (status, _body) = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer legacy-token")),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_api_tokens_override_legacy_token() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-authz-api-precedence")?;
        let mut cfg = sample_runtime_config(Some("legacy-token".to_string()));
        cfg.api.read_auth_token = Some("read-token".to_string());
        cfg.api.admin_auth_token = Some("admin-token".to_string());
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let (status, _body) = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer legacy-token")),
            None,
        )
        .await?;
        assert!(status.contains("401"), "expected 401, got: {status}");

        let (status, _body) = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer read-token")),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_returns_structured_json_and_since_filter(
    ) -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-verbose-json")?;
        let (mut ctx, _store) = build_ctx(None).await?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_debug_snapshot_subscriber(debug_subscriber);

        let (status, body) =
            send_plain_request(&mut ctx, format_get("/debug/verbose?since=1", None), None).await?;
        assert!(status.contains("200"), "expected 200, got: {status}");

        let decoded: serde_json::Value = serde_json::from_slice(&body).map_err(|err| {
            WorkerError::Message(format!("decode debug verbose json failed: {err}"))
        })?;
        assert_eq!(decoded["meta"]["schema_version"], "v1");
        assert_eq!(decoded["meta"]["sequence"], 2);
        assert!(decoded["timeline"].is_array());
        assert!(decoded["changes"].is_array());
        assert_eq!(
            decoded["changes"].as_array().map(|value| value.len()),
            Some(0)
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_snapshot_route_is_kept_for_backward_compatibility() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-snapshot-compat")?;
        let (mut ctx, _store) = build_ctx(None).await?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_debug_snapshot_subscriber(debug_subscriber);

        let (status, body) =
            send_plain_request(&mut ctx, format_get("/debug/snapshot", None), None).await?;
        assert!(status.contains("200"), "expected 200, got: {status}");
        let body_text = String::from_utf8(body)
            .map_err(|err| WorkerError::Message(format!("snapshot body not utf8: {err}")))?;
        assert!(body_text.contains("SystemSnapshot"));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_404_when_debug_disabled() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-disabled-404")?;
        let mut cfg = sample_runtime_config(None);
        cfg.debug.enabled = false;
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let (status, _body) =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert!(status.contains("404"), "expected 404, got: {status}");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_503_without_subscriber() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-missing-subscriber")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let (status, _body) =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert!(status.contains("503"), "expected 503, got: {status}");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_ui_route_returns_html_scaffold() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-ui-html")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let (status, body) =
            send_plain_request(&mut ctx, format_get("/debug/ui", None), None).await?;
        assert!(status.contains("200"), "expected 200, got: {status}");
        let html = String::from_utf8(body)
            .map_err(|err| WorkerError::Message(format!("ui body not utf8: {err}")))?;
        assert!(html.contains("id=\"meta-panel\""));
        assert!(html.contains("/debug/verbose"));
        assert!(html.contains("id=\"timeline-panel\""));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_routes_require_auth_when_tokens_set() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-authz")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_debug_snapshot_subscriber(debug_subscriber);

        let (status, _body) =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert!(status.contains("401"), "expected 401, got: {status}");

        let (status, _body) = send_plain_request(
            &mut ctx,
            format_get("/debug/verbose", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");

        let (status, _body) = send_plain_request(
            &mut ctx,
            format_get("/debug/ui", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");

        let (status, _body) = send_plain_request(
            &mut ctx,
            format_get("/debug/verbose", Some(&roles.admin_bearer_header())),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_disabled_accepts_plain_rejects_tls() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-disabled")?;
        let namespace = guard.namespace()?;

        let server_ca = generate_ca("server-ca-disabled")?;
        let server = generate_leaf_cert(
            "server-disabled",
            "localhost",
            false,
            &server_ca.issuer,
            false,
        )?;
        let _material = write_tls_material(
            namespace,
            "disabled",
            Some(server_ca.cert.cert_pem.as_bytes()),
            Some(server.cert_pem.as_bytes()),
            Some(server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;

        let (status, _body) =
            send_plain_request(&mut ctx, format_get("/fallback/cluster", None), None).await?;
        assert!(status.contains("200"), "expected 200, got: {status}");

        let trusted_client = build_client_config(&server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx, trusted_client, "localhost").await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_optional_accepts_plain_and_tls() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-optional")?;
        let namespace = guard.namespace()?;

        let server_ca = generate_ca("server-ca-optional")?;
        let server = generate_leaf_cert(
            "server-optional",
            "localhost",
            false,
            &server_ca.issuer,
            false,
        )?;
        let _material = write_tls_material(
            namespace,
            "optional",
            Some(server_ca.cert.cert_pem.as_bytes()),
            Some(server.cert_pem.as_bytes()),
            Some(server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Optional,
            Some(build_server_config(&server, &server_ca.cert)?),
        )?;

        let (status, _body) =
            send_plain_request(&mut ctx, format_get("/fallback/cluster", None), None).await?;
        assert!(status.contains("200"), "expected 200, got: {status}");

        let client_cfg = build_client_config(&server_ca.cert, None, None)?;
        let (status, _body) = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_required_accepts_tls_rejects_plain() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-required")?;
        let namespace = guard.namespace()?;

        let server_ca = generate_ca("server-ca-required")?;
        let server = generate_leaf_cert(
            "server-required",
            "localhost",
            false,
            &server_ca.issuer,
            false,
        )?;
        let _material = write_tls_material(
            namespace,
            "required",
            Some(server_ca.cert.cert_pem.as_bytes()),
            Some(server.cert_pem.as_bytes()),
            Some(server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(&server, &server_ca.cert)?),
        )?;

        let client_cfg = build_client_config(&server_ca.cert, None, None)?;
        let (status, _body) = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");

        let addr = ctx.local_addr()?;
        let mut plain = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
        plain
            .write_all(format_get("/fallback/cluster", None).as_bytes())
            .await
            .map_err(|err| WorkerError::Message(format!("plain write failed: {err}")))?;
        step_once(&mut ctx).await?;
        let mut raw = Vec::new();
        plain
            .read_to_end(&mut raw)
            .await
            .map_err(|err| WorkerError::Message(format!("plain read failed: {err}")))?;
        let response_text = String::from_utf8_lossy(&raw);
        assert!(
            !response_text.contains("HTTP/1.1 200"),
            "expected plaintext request rejection in required mode, got: {response_text}"
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_wrong_ca_and_hostname_and_expiry_failures() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-failures")?;
        let namespace = guard.namespace()?;

        let valid_ca = generate_ca("server-valid-ca")?;
        let wrong_ca = generate_ca("wrong-ca")?;
        let valid_server =
            generate_leaf_cert("server-valid", "localhost", false, &valid_ca.issuer, false)?;
        let expired_server =
            generate_leaf_cert("server-expired", "localhost", true, &valid_ca.issuer, false)?;

        let _material_valid = write_tls_material(
            namespace,
            "valid-server",
            Some(valid_ca.cert.cert_pem.as_bytes()),
            Some(valid_server.cert_pem.as_bytes()),
            Some(valid_server.key_pem.as_bytes()),
        )?;
        let _material_expired = write_tls_material(
            namespace,
            "expired-server",
            Some(valid_ca.cert.cert_pem.as_bytes()),
            Some(expired_server.cert_pem.as_bytes()),
            Some(expired_server.key_pem.as_bytes()),
        )?;

        let (mut ctx_wrong_ca, _store) = build_ctx(None).await?;
        ctx_wrong_ca.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(&valid_server, &valid_ca.cert)?),
        )?;
        let client_wrong_ca = build_client_config(&wrong_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_wrong_ca, client_wrong_ca, "localhost").await?;

        let (mut ctx_hostname, _store) = build_ctx(None).await?;
        ctx_hostname.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(&valid_server, &valid_ca.cert)?),
        )?;
        let client_hostname = build_client_config(&valid_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_hostname, client_hostname, "not-localhost").await?;

        let (mut ctx_expired, _store) = build_ctx(None).await?;
        ctx_expired.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(&expired_server, &valid_ca.cert)?),
        )?;
        let client_expired = build_client_config(&valid_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_expired, client_expired, "localhost").await?;

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_mtls_node_auth_allows_trusted_client_only() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-mtls-node-auth")?;
        let namespace = guard.namespace()?;

        let server_ca = generate_ca("mtls-server-ca")?;
        let trusted_client_ca = generate_ca("mtls-trusted-client-ca")?;
        let untrusted_client_ca = generate_ca("mtls-untrusted-client-ca")?;
        let server =
            generate_leaf_cert("mtls-server", "localhost", false, &server_ca.issuer, false)?;
        let trusted_client = generate_leaf_cert(
            "trusted-client",
            "localhost",
            false,
            &trusted_client_ca.issuer,
            true,
        )?;
        let untrusted_client = generate_leaf_cert(
            "untrusted-client",
            "localhost",
            false,
            &untrusted_client_ca.issuer,
            true,
        )?;

        let _material_server = write_tls_material(
            namespace,
            "mtls-server",
            Some(server_ca.cert.cert_pem.as_bytes()),
            Some(server.cert_pem.as_bytes()),
            Some(server.key_pem.as_bytes()),
        )?;
        let _material_trusted = write_tls_material(
            namespace,
            "mtls-trusted-client",
            Some(trusted_client_ca.cert.cert_pem.as_bytes()),
            Some(trusted_client.cert_pem.as_bytes()),
            Some(trusted_client.key_pem.as_bytes()),
        )?;
        let _material_untrusted = write_tls_material(
            namespace,
            "mtls-untrusted-client",
            Some(untrusted_client_ca.cert.cert_pem.as_bytes()),
            Some(untrusted_client.cert_pem.as_bytes()),
            Some(untrusted_client.key_pem.as_bytes()),
        )?;

        let mode = TlsMode::Required;
        assert!(matches!(mode, TlsMode::Required));

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config_with_client_auth(
                &server,
                &server_ca.cert,
                &trusted_client_ca.cert,
            )?),
        )?;
        ctx.set_require_client_cert(true);

        let trusted_cfg = build_client_config(
            &server_ca.cert,
            Some(&trusted_client),
            Some(&trusted_client_ca.cert),
        )?;
        let (status, _body) = send_tls_request(
            &mut ctx,
            trusted_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert!(status.contains("200"), "expected 200, got: {status}");

        let missing_client_cert_cfg = build_client_config(&server_ca.cert, None, None)?;
        expect_tls_request_rejected(&mut ctx, missing_client_cert_cfg, "localhost").await?;

        let untrusted_client_cfg = build_client_config(
            &server_ca.cert,
            Some(&untrusted_client),
            Some(&untrusted_client_ca.cert),
        )?;
        expect_tls_request_rejected(&mut ctx, untrusted_client_cfg, "localhost").await?;

        Ok(())
    }
}
