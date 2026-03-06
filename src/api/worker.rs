use std::{collections::BTreeMap, sync::Arc, time::Duration};

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
    config::{ApiAuthConfig, ApiTlsMode, RuntimeConfig},
    dcs::store::DcsStore,
    debug_api::{snapshot::SystemSnapshot, view::build_verbose_payload},
    logging::{EventMeta, LogHandle, SeverityText},
    state::{StateSubscriber, WorkerError},
};

#[derive(Clone, Debug, Default)]
struct ApiRoleTokens {
    read_token: Option<String>,
    admin_token: Option<String>,
}

pub struct ApiWorkerCtx {
    listener: TcpListener,
    poll_interval: Duration,
    scope: String,
    member_id: String,
    config_subscriber: StateSubscriber<RuntimeConfig>,
    dcs_store: Box<dyn DcsStore>,
    debug_snapshot_subscriber: Option<StateSubscriber<SystemSnapshot>>,
    tls_mode_override: Option<ApiTlsMode>,
    tls_acceptor: Option<TlsAcceptor>,
    role_tokens: Option<ApiRoleTokens>,
    require_client_cert: bool,
    log: LogHandle,
}

impl ApiWorkerCtx {
    pub fn contract_stub(
        listener: TcpListener,
        config_subscriber: StateSubscriber<RuntimeConfig>,
        dcs_store: Box<dyn DcsStore>,
    ) -> Self {
        Self::new(
            listener,
            config_subscriber,
            dcs_store,
            LogHandle::disabled(),
        )
    }

    pub(crate) fn new(
        listener: TcpListener,
        config_subscriber: StateSubscriber<RuntimeConfig>,
        dcs_store: Box<dyn DcsStore>,
        log: LogHandle,
    ) -> Self {
        let scope = config_subscriber.latest().value.dcs.scope.clone();
        let member_id = config_subscriber.latest().value.cluster.member_id.clone();
        Self {
            listener,
            poll_interval: Duration::from_millis(10),
            scope,
            member_id,
            config_subscriber,
            dcs_store,
            debug_snapshot_subscriber: None,
            tls_mode_override: None,
            tls_acceptor: None,
            role_tokens: None,
            require_client_cert: false,
            log,
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

    pub(crate) fn set_ha_snapshot_subscriber(
        &mut self,
        subscriber: StateSubscriber<SystemSnapshot>,
    ) {
        self.debug_snapshot_subscriber = Some(subscriber);
    }
}

pub async fn run(mut ctx: ApiWorkerCtx) -> Result<(), WorkerError> {
    loop {
        if let Err(err) = step_once(&mut ctx).await {
            let fatal = is_fatal_api_step_error(&err);
            let mut attrs = api_base_attrs(&ctx);
            attrs.insert(
                "error".to_string(),
                serde_json::Value::String(err.to_string()),
            );
            attrs.insert("fatal".to_string(), serde_json::Value::Bool(fatal));
            ctx.log
                .emit_event(
                    if fatal {
                        SeverityText::Error
                    } else {
                        SeverityText::Warn
                    },
                    "api step failed",
                    "api_worker::run",
                    EventMeta::new("api.step_once_failed", "api", "failed"),
                    attrs,
                )
                .map_err(|emit_err| {
                    WorkerError::Message(format!("api step failure log emit failed: {emit_err}"))
                })?;

            if fatal {
                return Err(err);
            }
        }
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub async fn step_once(ctx: &mut ApiWorkerCtx) -> Result<(), WorkerError> {
    let (stream, peer) =
        match tokio::time::timeout(Duration::from_millis(1), ctx.listener.accept()).await {
            Ok(Ok((stream, peer))) => (stream, peer),
            Ok(Err(err)) => {
                return Err(WorkerError::Message(format!("api accept failed: {err}")));
            }
            Err(_elapsed) => return Ok(()),
        };

    let cfg = ctx.config_subscriber.latest().value;
    let mut accept_attrs = api_base_attrs(ctx);
    accept_attrs.insert(
        "api.peer_addr".to_string(),
        serde_json::Value::String(peer.to_string()),
    );
    accept_attrs.insert(
        "api.tls_mode".to_string(),
        serde_json::Value::String(format!("{:?}", effective_tls_mode(ctx, &cfg)).to_lowercase()),
    );
    ctx.log
        .emit_event(
            SeverityText::Debug,
            "api connection accepted",
            "api_worker::step_once",
            EventMeta::new("api.connection_accepted", "api", "ok"),
            accept_attrs,
        )
        .map_err(|err| WorkerError::Message(format!("api accept log emit failed: {err}")))?;

    let mut stream = match accept_connection(ctx, &cfg, peer, stream).await? {
        Some(stream) => stream,
        None => return Ok(()),
    };

    let request =
        match tokio::time::timeout(Duration::from_millis(100), stream.read_http_request()).await {
            Ok(Ok(req)) => req,
            Ok(Err(message)) => {
                let mut attrs = api_base_attrs(ctx);
                attrs.insert(
                    "api.peer_addr".to_string(),
                    serde_json::Value::String(peer.to_string()),
                );
                attrs.insert(
                    "error".to_string(),
                    serde_json::Value::String(message.clone()),
                );
                ctx.log
                    .emit_event(
                        SeverityText::Warn,
                        "api request parse failed",
                        "api_worker::step_once",
                        EventMeta::new("api.request_parse_failed", "api", "failed"),
                        attrs,
                    )
                    .map_err(|err| {
                        WorkerError::Message(format!("api parse failure log emit failed: {err}"))
                    })?;
                let response = HttpResponse::text(400, "Bad Request", message);
                stream.write_http_response(response).await?;
                return Ok(());
            }
            Err(_elapsed) => return Ok(()),
        };

    match authorize_request(ctx, &cfg, &request) {
        AuthDecision::Allowed => {}
        AuthDecision::Unauthorized => {
            emit_api_auth_decision(ctx, peer, &request, "unauthorized")?;
            let response = HttpResponse::text(401, "Unauthorized", "unauthorized");
            stream.write_http_response(response).await?;
            return Ok(());
        }
        AuthDecision::Forbidden => {
            emit_api_auth_decision(ctx, peer, &request, "forbidden")?;
            let response = HttpResponse::text(403, "Forbidden", "forbidden");
            stream.write_http_response(response).await?;
            return Ok(());
        }
    }

    emit_api_auth_decision(ctx, peer, &request, "allowed")?;

    let response = route_request(ctx, &cfg, peer, request);
    let status_code = response.status;
    stream.write_http_response(response).await?;

    let mut attrs = api_base_attrs(ctx);
    attrs.insert(
        "api.peer_addr".to_string(),
        serde_json::Value::String(peer.to_string()),
    );
    attrs.insert(
        "api.status_code".to_string(),
        serde_json::Value::Number(serde_json::Number::from(status_code as u64)),
    );
    ctx.log
        .emit_event(
            SeverityText::Debug,
            "api response sent",
            "api_worker::step_once",
            EventMeta::new("api.response_sent", "api", "ok"),
            attrs,
        )
        .map_err(|err| WorkerError::Message(format!("api response log emit failed: {err}")))?;
    Ok(())
}

fn api_base_attrs(ctx: &ApiWorkerCtx) -> BTreeMap<String, serde_json::Value> {
    let mut attrs = BTreeMap::new();
    attrs.insert(
        "scope".to_string(),
        serde_json::Value::String(ctx.scope.clone()),
    );
    attrs.insert(
        "member_id".to_string(),
        serde_json::Value::String(ctx.member_id.clone()),
    );
    attrs
}

fn extract_request_id(request: &HttpRequest) -> Option<String> {
    request
        .headers
        .iter()
        .find(|(name, _value)| name.eq_ignore_ascii_case("x-request-id"))
        .map(|(_name, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.len() > 128 {
                value[..128].to_string()
            } else {
                value
            }
        })
}

fn auth_header_present(request: &HttpRequest) -> bool {
    request
        .headers
        .iter()
        .any(|(name, _value)| name.eq_ignore_ascii_case("authorization"))
}

fn route_template(request: &HttpRequest) -> String {
    let (path, _query) = split_path_and_query(&request.path);
    format!("{} {}", request.method, path)
}

fn emit_api_auth_decision(
    ctx: &ApiWorkerCtx,
    peer: std::net::SocketAddr,
    request: &HttpRequest,
    decision: &str,
) -> Result<(), WorkerError> {
    let mut attrs = api_base_attrs(ctx);
    attrs.insert(
        "api.peer_addr".to_string(),
        serde_json::Value::String(peer.to_string()),
    );
    attrs.insert(
        "api.method".to_string(),
        serde_json::Value::String(request.method.clone()),
    );
    attrs.insert(
        "api.route_template".to_string(),
        serde_json::Value::String(route_template(request)),
    );
    attrs.insert(
        "api.auth.header_present".to_string(),
        serde_json::Value::Bool(auth_header_present(request)),
    );
    attrs.insert(
        "api.auth.result".to_string(),
        serde_json::Value::String(decision.to_string()),
    );
    attrs.insert(
        "api.auth.required_role".to_string(),
        serde_json::Value::String(format!("{:?}", endpoint_role(request)).to_lowercase()),
    );
    if let Some(request_id) = extract_request_id(request) {
        attrs.insert(
            "api.request_id".to_string(),
            serde_json::Value::String(request_id),
        );
    }
    ctx.log
        .emit_event(
            SeverityText::Debug,
            "api auth decision",
            "api_worker::authorize_request",
            EventMeta::new("api.auth_decision", "api", "ok"),
            attrs,
        )
        .map_err(|err| WorkerError::Message(format!("api auth log emit failed: {err}")))?;
    Ok(())
}

fn is_fatal_api_step_error(err: &WorkerError) -> bool {
    let message = err.to_string();
    message.contains("api accept failed")
        || message.contains("tls mode requires a configured tls acceptor")
        || message.contains("api local_addr failed")
}

fn route_request(
    ctx: &mut ApiWorkerCtx,
    cfg: &RuntimeConfig,
    _peer: std::net::SocketAddr,
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
        ["decision", payload.ha.decision],
        ["decision_detail", payload.ha.decision_detail ?? "<none>"],
        ["planned_actions", payload.ha.planned_actions]
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

    match &cfg.api.security.auth {
        ApiAuthConfig::Disabled => ApiRoleTokens {
            read_token: None,
            admin_token: None,
        },
        ApiAuthConfig::RoleTokens(tokens) => ApiRoleTokens {
            read_token: normalize_runtime_token(tokens.read_token.clone()),
            admin_token: normalize_runtime_token(tokens.admin_token.clone()),
        },
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
    peer: std::net::SocketAddr,
    stream: TcpStream,
) -> Result<Option<ApiConnection>, WorkerError> {
    match effective_tls_mode(ctx, cfg) {
        ApiTlsMode::Disabled => Ok(Some(ApiConnection::Plain(stream))),
        ApiTlsMode::Required => {
            let acceptor = require_tls_acceptor(ctx)?;
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    if ctx.require_client_cert && !has_peer_client_cert(&tls_stream) {
                        let mut attrs = api_base_attrs(ctx);
                        attrs.insert(
                            "api.peer_addr".to_string(),
                            serde_json::Value::String(peer.to_string()),
                        );
                        attrs.insert(
                            "api.tls_mode".to_string(),
                            serde_json::Value::String("required".to_string()),
                        );
                        ctx.log
                            .emit_event(
                                SeverityText::Warn,
                                "tls client cert missing",
                                "api_worker::accept_connection",
                                EventMeta::new("api.tls_client_cert_missing", "api", "failed"),
                                attrs,
                            )
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "api tls missing cert log emit failed: {err}"
                                ))
                            })?;
                        return Ok(None);
                    }
                    Ok(Some(ApiConnection::Tls(Box::new(tls_stream))))
                }
                Err(err) => {
                    let mut attrs = api_base_attrs(ctx);
                    attrs.insert(
                        "api.peer_addr".to_string(),
                        serde_json::Value::String(peer.to_string()),
                    );
                    attrs.insert(
                        "api.tls_mode".to_string(),
                        serde_json::Value::String("required".to_string()),
                    );
                    attrs.insert(
                        "error".to_string(),
                        serde_json::Value::String(err.to_string()),
                    );
                    ctx.log
                        .emit_event(
                            SeverityText::Warn,
                            "tls handshake failed",
                            "api_worker::accept_connection",
                            EventMeta::new("api.tls_handshake_failed", "api", "failed"),
                            attrs,
                        )
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "api tls handshake log emit failed: {emit_err}"
                            ))
                        })?;
                    Ok(None)
                }
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
                        let mut attrs = api_base_attrs(ctx);
                        attrs.insert(
                            "api.peer_addr".to_string(),
                            serde_json::Value::String(peer.to_string()),
                        );
                        attrs.insert(
                            "api.tls_mode".to_string(),
                            serde_json::Value::String("optional".to_string()),
                        );
                        ctx.log
                            .emit_event(
                                SeverityText::Warn,
                                "tls client cert missing",
                                "api_worker::accept_connection",
                                EventMeta::new("api.tls_client_cert_missing", "api", "failed"),
                                attrs,
                            )
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "api tls missing cert log emit failed: {err}"
                                ))
                            })?;
                        return Ok(None);
                    }
                    Ok(Some(ApiConnection::Tls(Box::new(tls_stream))))
                }
                Err(err) => {
                    let mut attrs = api_base_attrs(ctx);
                    attrs.insert(
                        "api.peer_addr".to_string(),
                        serde_json::Value::String(peer.to_string()),
                    );
                    attrs.insert(
                        "api.tls_mode".to_string(),
                        serde_json::Value::String("optional".to_string()),
                    );
                    attrs.insert(
                        "error".to_string(),
                        serde_json::Value::String(err.to_string()),
                    );
                    ctx.log
                        .emit_event(
                            SeverityText::Warn,
                            "tls handshake failed",
                            "api_worker::accept_connection",
                            EventMeta::new("api.tls_handshake_failed", "api", "failed"),
                            attrs,
                        )
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "api tls handshake log emit failed: {emit_err}"
                            ))
                        })?;
                    Ok(None)
                }
            }
        }
    }
}

fn effective_tls_mode(ctx: &ApiWorkerCtx, cfg: &RuntimeConfig) -> ApiTlsMode {
    if let Some(mode) = ctx.tls_mode_override {
        return mode;
    }

    cfg.api.security.tls.mode
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
    match status {
        httparse::Status::Complete(_bytes) => {}
        httparse::Status::Partial => return Ok(None),
    }

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
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use serde_json::Value;

    use rustls::{pki_types::ServerName, ClientConfig};
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio_rustls::TlsConnector;

    use crate::logging::{LogHandle, LogSink, SeverityText, TestSink};
    use crate::pginfo::conninfo::PgSslMode;
    use crate::{
        api::worker::{step_once, ApiWorkerCtx},
        config::{
            ApiAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiSecurityConfig, ApiTlsMode,
            BinaryPaths, ClusterConfig, DcsConfig, DebugConfig, HaConfig, InlineOrPath,
            LogCleanupConfig, LogLevel, LoggingConfig, PgHbaConfig, PgIdentConfig, PostgresConfig,
            PostgresConnIdentityConfig, PostgresLoggingConfig, PostgresRoleConfig,
            PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig, StderrSinkConfig,
            TlsServerConfig,
        },
        dcs::state::{DcsCache, DcsState, DcsTrust},
        dcs::store::{DcsStore, DcsStoreError, WatchEvent},
        debug_api::snapshot::{
            AppLifecycle, DebugChangeEvent, DebugDomain, DebugTimelineEntry, SystemSnapshot,
        },
        ha::{
            decision::HaDecision,
            state::{HaPhase, HaState},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, UnixMillis, WorkerError},
        test_harness::{
            auth::ApiRoleTokens,
            namespace::NamespaceGuard,
            tls::{
                build_adversarial_tls_fixture, build_client_config, build_server_config,
                build_server_config_with_client_auth, write_tls_material,
            },
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

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(())
        }

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(true)
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

    fn sample_runtime_config(auth_token: Option<String>) -> RuntimeConfig {
        let auth = match auth_token {
            Some(token) => ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
                read_token: Some(token.clone()),
                admin_token: Some(token),
            }),
            None => ApiAuthConfig::Disabled,
        };

        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: "/tmp/pgdata".into(),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: "/tmp/pgtuskmaster/socket".into(),
                log_file: "/tmp/pgtuskmaster/postgres.log".into(),
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: "local all all trust\n".to_string(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: "# empty\n".to_string(),
                    },
                },
                extra_gucs: std::collections::BTreeMap::new(),
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
                init: None,
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
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                        protect_recent_seconds: 300,
                    },
                },
                sinks: crate::config::LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: crate::config::FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: crate::config::FileSinkMode::Append,
                    },
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:0".to_string(),
                security: ApiSecurityConfig {
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    auth,
                },
            },
            debug: DebugConfig { enabled: true },
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
            decision: HaDecision::EnterFailSafe {
                release_leader_lease: false,
            },
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

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
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

    async fn build_ctx_with_config_and_log(
        cfg: RuntimeConfig,
    ) -> Result<(ApiWorkerCtx, RecordingStore, Arc<TestSink>), WorkerError> {
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

        let store = RecordingStore::default();
        let (log, sink) = test_log_handle();
        let ctx = ApiWorkerCtx::new(listener, cfg_subscriber, Box::new(store.clone()), log);
        Ok((ctx, store, sink))
    }

    async fn build_ctx(
        auth_token: Option<String>,
    ) -> Result<(ApiWorkerCtx, RecordingStore), WorkerError> {
        build_ctx_with_config(sample_runtime_config(auth_token)).await
    }

    const HEADER_LIMIT: usize = 16 * 1024;
    const MAX_BODY_BYTES: usize = 256 * 1024;
    const MAX_RESPONSE_BYTES: usize = HEADER_LIMIT + MAX_BODY_BYTES;
    const IO_TIMEOUT: Duration = Duration::from_secs(2);

    #[derive(Debug)]
    struct TestHttpResponse {
        status_code: u16,
        body: Vec<u8>,
    }

    #[derive(Debug)]
    struct ParsedHttpHead {
        status_code: u16,
        content_length: usize,
        body_start: usize,
    }

    fn parse_http_response_head(
        raw: &[u8],
        header_end: usize,
    ) -> Result<ParsedHttpHead, WorkerError> {
        let head = raw.get(..header_end).ok_or_else(|| {
            WorkerError::Message("response header end offset out of bounds".to_string())
        })?;

        let status_line_end = head
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| WorkerError::Message("response missing status line".to_string()))?;

        let status_line_bytes = head.get(..status_line_end).ok_or_else(|| {
            WorkerError::Message("response status line offset out of bounds".to_string())
        })?;
        let status_line = std::str::from_utf8(status_line_bytes)
            .map_err(|err| WorkerError::Message(format!("response status line not utf8: {err}")))?;

        let mut status_parts = status_line.split_whitespace();
        let http_version = status_parts.next().ok_or_else(|| {
            WorkerError::Message("response status line missing http version".to_string())
        })?;
        if http_version != "HTTP/1.1" {
            return Err(WorkerError::Message(format!(
                "unexpected http version in response: {http_version}"
            )));
        }
        let status_str = status_parts.next().ok_or_else(|| {
            WorkerError::Message("response status line missing status code".to_string())
        })?;
        if status_str.len() != 3 || !status_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(WorkerError::Message(format!(
                "response status code must be 3 digits, got: {status_str}"
            )));
        }
        let status_code = status_str.parse::<u16>().map_err(|err| {
            WorkerError::Message(format!("response status code parse failed: {err}"))
        })?;
        if !(100..=599).contains(&status_code) {
            return Err(WorkerError::Message(format!(
                "response status code out of range: {status_code}"
            )));
        }

        let header_text = head.get(status_line_end + 2..).ok_or_else(|| {
            WorkerError::Message("response header offset out of bounds".to_string())
        })?;
        let header_text = std::str::from_utf8(header_text)
            .map_err(|err| WorkerError::Message(format!("response headers not utf8: {err}")))?;

        let mut content_length: Option<usize> = None;
        for line in header_text.split("\r\n") {
            if line.is_empty() {
                continue;
            }
            let (name, value) = line.split_once(':').ok_or_else(|| {
                WorkerError::Message(format!(
                    "invalid response header line (missing ':'): {line}"
                ))
            })?;
            if name.trim().eq_ignore_ascii_case("Content-Length") {
                if content_length.is_some() {
                    return Err(WorkerError::Message(
                        "response contains multiple Content-Length headers".to_string(),
                    ));
                }
                let parsed = value.trim().parse::<usize>().map_err(|err| {
                    WorkerError::Message(format!("response Content-Length parse failed: {err}"))
                })?;
                content_length = Some(parsed);
            }
        }

        let content_length = content_length.ok_or_else(|| {
            WorkerError::Message("response missing Content-Length header".to_string())
        })?;

        let body_start = header_end
            .checked_add(4)
            .ok_or_else(|| WorkerError::Message("response body offset overflow".to_string()))?;

        Ok(ParsedHttpHead {
            status_code,
            content_length,
            body_start,
        })
    }

    async fn read_http_response_framed(
        stream: &mut (impl AsyncRead + Unpin),
        timeout: Duration,
    ) -> Result<TestHttpResponse, WorkerError> {
        let response = tokio::time::timeout(timeout, async {
            let mut raw: Vec<u8> = Vec::new();
            let mut scratch = [0u8; 4096];

            let mut parsed_head: Option<ParsedHttpHead> = None;
            let mut expected_total_len: Option<usize> = None;

            loop {
                if let Some(expected) = expected_total_len {
                    if raw.len() == expected {
                        let parsed = parsed_head.ok_or_else(|| {
                            WorkerError::Message("response framing parsed without header".to_string())
                        })?;
                        let body = raw
                            .get(parsed.body_start..expected)
                            .ok_or_else(|| {
                                WorkerError::Message(
                                    "response body slice out of bounds after framing".to_string(),
                                )
                            })?
                            .to_vec();
                        return Ok(TestHttpResponse {
                            status_code: parsed.status_code,
                            body,
                        });
                    }
                    if raw.len() > expected {
                        return Err(WorkerError::Message(format!(
                            "response exceeded expected length (expected {expected} bytes, got {})",
                            raw.len()
                        )));
                    }
                } else {
                    if raw.len() > HEADER_LIMIT {
                        return Err(WorkerError::Message(format!(
                            "response headers exceeded limit of {HEADER_LIMIT} bytes"
                        )));
                    }

                    if let Some(header_end) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                        let head = parse_http_response_head(&raw, header_end)?;
                        if head.content_length > MAX_BODY_BYTES {
                            return Err(WorkerError::Message(format!(
                                "response body exceeded limit of {MAX_BODY_BYTES} bytes (Content-Length={})",
                                head.content_length
                            )));
                        }
                        let expected =
                            head.body_start.checked_add(head.content_length).ok_or_else(|| {
                                WorkerError::Message("response total length overflow".to_string())
                            })?;
                        if expected > MAX_RESPONSE_BYTES {
                            return Err(WorkerError::Message(format!(
                                "response exceeded limit of {MAX_RESPONSE_BYTES} bytes (expected {expected})"
                            )));
                        }
                        parsed_head = Some(head);
                        expected_total_len = Some(expected);
                        continue;
                    }
                }

                let n = stream.read(&mut scratch).await.map_err(|err| {
                    WorkerError::Message(format!("client read failed: {err}"))
                })?;
                if n == 0 {
                    return Err(WorkerError::Message(format!(
                        "unexpected eof while reading response (read {} bytes so far)",
                        raw.len()
                    )));
                }

                let new_len = raw.len().checked_add(n).ok_or_else(|| {
                    WorkerError::Message("response length overflow while reading".to_string())
                })?;
                if new_len > MAX_RESPONSE_BYTES {
                    return Err(WorkerError::Message(format!(
                        "response exceeded limit of {MAX_RESPONSE_BYTES} bytes while reading (would reach {new_len})"
                    )));
                }
                raw.extend_from_slice(&scratch[..n]);
            }
        })
        .await;

        match response {
            Ok(inner) => inner,
            Err(_) => Err(WorkerError::Message(format!(
                "timed out reading framed http response after {}s",
                timeout.as_secs()
            ))),
        }
    }

    async fn send_plain_request(
        ctx: &mut ApiWorkerCtx,
        request_head: String,
        body: Option<Vec<u8>>,
    ) -> Result<TestHttpResponse, WorkerError> {
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
        read_http_response_framed(&mut client, IO_TIMEOUT).await
    }

    async fn send_tls_request(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
        request_head: String,
        body: Option<Vec<u8>>,
    ) -> Result<TestHttpResponse, WorkerError> {
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
            read_http_response_framed(&mut tls, IO_TIMEOUT).await
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
            Ok(response) => {
                if response.status_code == 200 {
                    Err(WorkerError::Message(format!(
                        "expected tls request rejection, got status {}",
                        response.status_code
                    )))
                } else {
                    Ok(())
                }
            }
            Err(_) => Ok(()),
        }
    }

    fn format_get(path: &str, auth: Option<&str>) -> String {
        match auth {
            Some(auth_header) => format!(
                "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: {auth_header}\r\n\r\n"
            ),
            None => format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
        }
    }

    fn format_post(path: &str, auth: Option<&str>, body: &[u8]) -> String {
        match auth {
            Some(auth_header) => format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: {auth_header}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            ),
            None => format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            ),
        }
    }

    fn format_delete(path: &str, auth: Option<&str>) -> String {
        match auth {
            Some(auth_header) => format!(
                "DELETE {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: {auth_header}\r\n\r\n"
            ),
            None => format!("DELETE {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
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

        let response = send_plain_request(
            &mut ctx,
            format_get("/fallback/cluster", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let post_body = br#"{"requested_by":"node-a"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/switchover",
                Some(&roles.read_bearer_header()),
                post_body.as_slice(),
            ),
            Some(post_body),
        )
        .await?;
        assert_eq!(response.status_code, 403);
        assert_eq!(store.write_count()?, 0);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn auth_decision_logs_do_not_leak_bearer_token() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-auth-redaction")?;

        let (mut ctx, _store, sink) =
            build_ctx_with_config_and_log(sample_runtime_config(None)).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let secret = "super-secret-token-value";
        let auth_header = format!("Bearer {secret}");
        let response = send_plain_request(
            &mut ctx,
            format_get("/fallback/cluster", Some(auth_header.as_str())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 401);

        let records = sink
            .snapshot()
            .map_err(|err| WorkerError::Message(format!("log snapshot failed: {err}")))?;

        let auth_decision_present = records.iter().any(|record| {
            matches!(
                record.attributes.get("event.name"),
                Some(Value::String(name)) if name == "api.auth_decision"
            )
        });
        if !auth_decision_present {
            return Err(WorkerError::Message(
                "expected api.auth_decision log event, but it was not emitted".to_string(),
            ));
        }

        for record in records {
            let encoded = serde_json::to_string(&record)
                .map_err(|err| WorkerError::Message(format!("encode log record failed: {err}")))?;
            if encoded.contains(secret) {
                return Err(WorkerError::Message(
                    "bearer token leaked into structured logs".to_string(),
                ));
            }
        }

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
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/switchover",
                Some(&roles.admin_bearer_header()),
                post_body.as_slice(),
            ),
            Some(post_body),
        )
        .await?;
        assert_eq!(response.status_code, 202);
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

        let response = send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert_eq!(response.status_code, 200);
        let decoded: serde_json::Value = serde_json::from_slice(&response.body)
            .map_err(|err| WorkerError::Message(format!("decode ha state json failed: {err}")))?;
        assert_eq!(decoded["cluster_name"], "cluster-a");
        assert_eq!(decoded["scope"], "scope-a");
        assert_eq!(decoded["self_member_id"], "node-a");
        assert_eq!(decoded["leader"], serde_json::Value::Null);
        assert_eq!(decoded["switchover_requested_by"], serde_json::Value::Null);
        assert_eq!(decoded["member_count"], 0);
        assert_eq!(decoded["dcs_trust"], "full_quorum");
        assert_eq!(decoded["ha_phase"], "replica");
        assert_eq!(decoded["ha_tick"], 7);
        assert_eq!(decoded["ha_decision"]["kind"], "enter_fail_safe");
        assert_eq!(decoded["ha_decision"]["release_leader_lease"], false);
        assert_eq!(decoded["snapshot_sequence"], 2);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_state_route_returns_503_without_subscriber() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-state-missing-subscriber")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let response = send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert_eq!(response.status_code, 503);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_leader_routes_are_not_found_and_do_not_mutate_dcs_keys() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-leader-routes-removed")?;
        let (mut ctx, store) = build_ctx(None).await?;

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post("/ha/leader", None, body.as_slice()),
            Some(body),
        )
        .await?;
        assert_eq!(response.status_code, 404);

        let response =
            send_plain_request(&mut ctx, format_delete("/ha/leader", None), None).await?;
        assert_eq!(response.status_code, 404);

        let response =
            send_plain_request(&mut ctx, format_delete("/ha/switchover", None), None).await?;
        assert_eq!(response.status_code, 202);

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

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/ha/leader",
                Some(&roles.read_bearer_header()),
                body.as_slice(),
            ),
            Some(body),
        )
        .await?;
        assert_eq!(response.status_code, 404);

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/ha/leader",
                Some(&roles.admin_bearer_header()),
                body.as_slice(),
            ),
            Some(body),
        )
        .await?;
        assert_eq!(response.status_code, 404);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_legacy_auth_token_fallback_protects_ha_routes() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-authz-legacy-fallback")?;
        let (mut ctx, _store) = build_ctx(Some("legacy-token".to_string())).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert_eq!(response.status_code, 401);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer legacy-token")),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_api_tokens_override_legacy_token() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-authz-api-precedence")?;
        let mut cfg = sample_runtime_config(Some("legacy-token".to_string()));
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some("read-token".to_string()),
            admin_token: Some("admin-token".to_string()),
        });
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer legacy-token")),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 401);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer read-token")),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_returns_structured_json_and_since_filter(
    ) -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-verbose-json")?;
        let (mut ctx, _store) = build_ctx(None).await?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose?since=1", None), None).await?;
        assert_eq!(response.status_code, 200);

        let decoded: serde_json::Value = serde_json::from_slice(&response.body).map_err(|err| {
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
        let endpoints = decoded["api"]["endpoints"].as_array().ok_or_else(|| {
            WorkerError::Message("debug verbose payload missing api.endpoints".to_string())
        })?;
        let contains_restore_route = endpoints.iter().any(|value| {
            value
                .as_str()
                .map(|route| route.contains("restore"))
                .unwrap_or(false)
        });
        assert!(!contains_restore_route);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_snapshot_route_is_kept_for_backward_compatibility() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-snapshot-compat")?;
        let (mut ctx, _store) = build_ctx(None).await?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response =
            send_plain_request(&mut ctx, format_get("/debug/snapshot", None), None).await?;
        assert_eq!(response.status_code, 200);
        let body_text = String::from_utf8(response.body)
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
        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert_eq!(response.status_code, 404);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_503_without_subscriber() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-missing-subscriber")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert_eq!(response.status_code, 503);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_ui_route_returns_html_scaffold() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-ui-html")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let response = send_plain_request(&mut ctx, format_get("/debug/ui", None), None).await?;
        assert_eq!(response.status_code, 200);
        let html = String::from_utf8(response.body)
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
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert_eq!(response.status_code, 401);

        let response = send_plain_request(
            &mut ctx,
            format_get("/debug/verbose", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let response = send_plain_request(
            &mut ctx,
            format_get("/debug/ui", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let response = send_plain_request(
            &mut ctx,
            format_get("/debug/verbose", Some(&roles.admin_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_disabled_accepts_plain_rejects_tls() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-disabled")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "disabled",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;

        let response =
            send_plain_request(&mut ctx, format_get("/fallback/cluster", None), None).await?;
        assert_eq!(response.status_code, 200);

        let trusted_client = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx, trusted_client, "localhost").await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_optional_accepts_plain_and_tls() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-optional")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "optional",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Optional,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;

        let response =
            send_plain_request(&mut ctx, format_get("/fallback/cluster", None), None).await?;
        assert_eq!(response.status_code, 200);

        let client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let response = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_required_accepts_tls_rejects_plain() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-required")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "required",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;

        let client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let response = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let addr = ctx.local_addr()?;
        let mut plain = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
        plain
            .write_all(format_get("/fallback/cluster", None).as_bytes())
            .await
            .map_err(|err| WorkerError::Message(format!("plain write failed: {err}")))?;
        step_once(&mut ctx).await?;
        let plain_result = read_http_response_framed(&mut plain, IO_TIMEOUT).await;
        if let Ok(plain_response) = plain_result {
            assert_ne!(plain_response.status_code, 200);
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_required_accepts_tls_with_production_tls_builder(
    ) -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-required-prod-builder")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "required-prod-builder",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let tls_cfg = crate::config::TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(crate::config::TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: fixture.valid_server.key_pem.clone(),
                },
            }),
            client_auth: None,
        };

        let server_cfg = crate::tls::build_rustls_server_config(&tls_cfg).map_err(|err| {
            WorkerError::Message(format!(
                "build production rustls server config failed: {err}"
            ))
        })?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(ApiTlsMode::Required, server_cfg)?;

        let client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let response = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_mtls_required_works_with_production_tls_builder() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-mtls-required-prod-builder")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material_server = write_tls_material(
            namespace,
            "mtls-server-prod-builder",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;
        let _material_trusted = write_tls_material(
            namespace,
            "mtls-trusted-client-prod-builder",
            Some(fixture.trusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.trusted_client.cert_pem.as_bytes()),
            Some(fixture.trusted_client.key_pem.as_bytes()),
        )?;
        let _material_untrusted = write_tls_material(
            namespace,
            "mtls-untrusted-client-prod-builder",
            Some(fixture.untrusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.key_pem.as_bytes()),
        )?;

        let tls_cfg = crate::config::TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(crate::config::TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: fixture.valid_server.key_pem.clone(),
                },
            }),
            client_auth: Some(crate::config::TlsClientAuthConfig {
                client_ca: InlineOrPath::Inline {
                    content: fixture.trusted_client_ca.cert.cert_pem.clone(),
                },
                require_client_cert: true,
            }),
        };

        let server_cfg = crate::tls::build_rustls_server_config(&tls_cfg).map_err(|err| {
            WorkerError::Message(format!(
                "build production rustls server config failed: {err}"
            ))
        })?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(ApiTlsMode::Required, server_cfg)?;
        ctx.set_require_client_cert(true);

        let trusted_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.trusted_client),
            Some(&fixture.trusted_client_ca.cert),
        )?;
        let response = send_tls_request(
            &mut ctx,
            trusted_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let missing_client_cert_cfg =
            build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_request_rejected(&mut ctx, missing_client_cert_cfg, "localhost").await?;

        let untrusted_client_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.untrusted_client),
            Some(&fixture.untrusted_client_ca.cert),
        )?;
        expect_tls_request_rejected(&mut ctx, untrusted_client_cfg, "localhost").await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_wrong_ca_and_hostname_and_expiry_failures() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-failures")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material_valid = write_tls_material(
            namespace,
            "valid-server",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;
        let _material_expired = write_tls_material(
            namespace,
            "expired-server",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.expired_server.cert_pem.as_bytes()),
            Some(fixture.expired_server.key_pem.as_bytes()),
        )?;

        let (mut ctx_wrong_ca, _store) = build_ctx(None).await?;
        ctx_wrong_ca.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;
        let client_wrong_ca = build_client_config(&fixture.wrong_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_wrong_ca, client_wrong_ca, "localhost").await?;

        let (mut ctx_hostname, _store) = build_ctx(None).await?;
        ctx_hostname.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;
        let client_hostname = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_hostname, client_hostname, "not-localhost").await?;

        let (mut ctx_expired, _store) = build_ctx(None).await?;
        ctx_expired.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.expired_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;
        let client_expired = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_expired, client_expired, "localhost").await?;

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_mtls_node_auth_allows_trusted_client_only() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-mtls-node-auth")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material_server = write_tls_material(
            namespace,
            "mtls-server",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;
        let _material_trusted = write_tls_material(
            namespace,
            "mtls-trusted-client",
            Some(fixture.trusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.trusted_client.cert_pem.as_bytes()),
            Some(fixture.trusted_client.key_pem.as_bytes()),
        )?;
        let _material_untrusted = write_tls_material(
            namespace,
            "mtls-untrusted-client",
            Some(fixture.untrusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config_with_client_auth(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
                &fixture.trusted_client_ca.cert,
            )?),
        )?;
        ctx.set_require_client_cert(true);

        let trusted_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.trusted_client),
            Some(&fixture.trusted_client_ca.cert),
        )?;
        let response = send_tls_request(
            &mut ctx,
            trusted_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let missing_client_cert_cfg =
            build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_request_rejected(&mut ctx, missing_client_cert_cfg, "localhost").await?;

        let untrusted_client_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.untrusted_client),
            Some(&fixture.untrusted_client_ca.cert),
        )?;
        expect_tls_request_rejected(&mut ctx, untrusted_client_cfg, "localhost").await?;

        Ok(())
    }
}
