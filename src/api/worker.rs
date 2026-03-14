use std::{sync::Arc, time::Duration};

use rustls::ServerConfig;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{server::TlsStream, TlsAcceptor};

use crate::{
    api::{
        controller::{
            build_node_state, delete_switchover, post_switchover, SwitchoverRequestInput,
        },
        ApiError,
    },
    config::{ApiAuthConfig, ApiTlsMode, RuntimeConfig},
    dcs::{DcsHandle, DcsView},
    ha::state::HaState,
    logging::{AppEvent, AppEventHeader, LogHandle, SeverityText, StructuredFields},
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{StateSubscriber, WorkerError},
};

const API_LOOP_POLL_INTERVAL: Duration = Duration::from_millis(10);
const API_ACCEPT_TIMEOUT: Duration = Duration::from_millis(1);
const API_REQUEST_READ_TIMEOUT: Duration = Duration::from_millis(100);
const API_TLS_CLIENT_HELLO_PEEK_TIMEOUT: Duration = Duration::from_millis(10);
const API_REQUEST_ID_MAX_LEN: usize = 128;
const HTTP_REQUEST_MAX_BYTES: usize = 1024 * 1024;
const HTTP_REQUEST_HEADER_LIMIT_BYTES: usize = 16 * 1024;
const HTTP_REQUEST_SCRATCH_BUFFER_BYTES: usize = 4096;
const HTTP_REQUEST_HEADER_CAPACITY: usize = 64;

#[derive(Clone, Debug, Default)]
struct ApiRoleTokens {
    read_token: Option<String>,
    admin_token: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum ApiEventKind {
    StepOnceFailed,
    ConnectionAccepted,
    RequestParseFailed,
    ResponseSent,
    AuthDecision,
    TlsClientCertMissing,
    TlsHandshakeFailed,
}

impl ApiEventKind {
    fn name(self) -> &'static str {
        match self {
            Self::StepOnceFailed => "api.step_once_failed",
            Self::ConnectionAccepted => "api.connection_accepted",
            Self::RequestParseFailed => "api.request_parse_failed",
            Self::ResponseSent => "api.response_sent",
            Self::AuthDecision => "api.auth_decision",
            Self::TlsClientCertMissing => "api.tls_client_cert_missing",
            Self::TlsHandshakeFailed => "api.tls_handshake_failed",
        }
    }
}

fn api_event(
    kind: ApiEventKind,
    result: &str,
    severity: SeverityText,
    message: impl Into<String>,
) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(kind.name(), "api", result),
    )
}

pub struct ApiWorkerCtx {
    listener: TcpListener,
    poll_interval: Duration,
    scope: String,
    member_id: String,
    config_subscriber: StateSubscriber<RuntimeConfig>,
    dcs_handle: DcsHandle,
    pg_subscriber: Option<StateSubscriber<PgInfoState>>,
    process_subscriber: Option<StateSubscriber<ProcessState>>,
    dcs_subscriber: Option<StateSubscriber<DcsView>>,
    ha_subscriber: Option<StateSubscriber<HaState>>,
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
        dcs_handle: DcsHandle,
    ) -> Self {
        Self::new(listener, config_subscriber, dcs_handle, LogHandle::disabled())
    }

    pub(crate) fn new(
        listener: TcpListener,
        config_subscriber: StateSubscriber<RuntimeConfig>,
        dcs_handle: DcsHandle,
        log: LogHandle,
    ) -> Self {
        let cfg = config_subscriber.latest();
        let scope = cfg.dcs.scope.clone();
        let member_id = cfg.cluster.member_id.clone();
        Self {
            listener,
            poll_interval: API_LOOP_POLL_INTERVAL,
            scope,
            member_id,
            config_subscriber,
            dcs_handle,
            pg_subscriber: None,
            process_subscriber: None,
            dcs_subscriber: None,
            ha_subscriber: None,
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

    pub fn set_require_client_cert(&mut self, required: bool) {
        self.require_client_cert = required;
    }

    pub(crate) fn set_live_state_subscribers(
        &mut self,
        pg_subscriber: StateSubscriber<PgInfoState>,
        process_subscriber: StateSubscriber<ProcessState>,
        dcs_subscriber: StateSubscriber<DcsView>,
        ha_subscriber: StateSubscriber<HaState>,
    ) {
        self.pg_subscriber = Some(pg_subscriber);
        self.process_subscriber = Some(process_subscriber);
        self.dcs_subscriber = Some(dcs_subscriber);
        self.ha_subscriber = Some(ha_subscriber);
    }
}

pub async fn run(mut ctx: ApiWorkerCtx) -> Result<(), WorkerError> {
    loop {
        if let Err(err) = step_once(&mut ctx).await {
            let fatal = is_fatal_api_step_error(&err);
            let mut event = api_event(
                ApiEventKind::StepOnceFailed,
                "failed",
                if fatal {
                    SeverityText::Error
                } else {
                    SeverityText::Warn
                },
                "api step failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(api_base_fields(&ctx).into_attributes());
            fields.insert("error", err.to_string());
            fields.insert("fatal", fatal);
            ctx.log
                .emit_app_event("api_worker::run", event)
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
    let (stream, peer) = match tokio::time::timeout(API_ACCEPT_TIMEOUT, ctx.listener.accept()).await
    {
        Ok(Ok((stream, peer))) => (stream, peer),
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!("api accept failed: {err}")));
        }
        Err(_elapsed) => return Ok(()),
    };

    let cfg = ctx.config_subscriber.latest();
    let mut accept_event = api_event(
        ApiEventKind::ConnectionAccepted,
        "ok",
        SeverityText::Debug,
        "api connection accepted",
    );
    let fields = accept_event.fields_mut();
    fields.append_json_map(api_base_fields(ctx).into_attributes());
    fields.insert("api.peer_addr", peer.to_string());
    fields.insert(
        "api.tls_mode",
        format!("{:?}", effective_tls_mode(ctx, &cfg)).to_lowercase(),
    );
    ctx.log
        .emit_app_event("api_worker::step_once", accept_event)
        .map_err(|err| WorkerError::Message(format!("api accept log emit failed: {err}")))?;

    let mut stream = match accept_connection(ctx, &cfg, peer, stream).await? {
        Some(stream) => stream,
        None => return Ok(()),
    };

    let request =
        match tokio::time::timeout(API_REQUEST_READ_TIMEOUT, stream.read_http_request()).await {
            Ok(Ok(req)) => req,
            Ok(Err(message)) => {
                let mut event = api_event(
                    ApiEventKind::RequestParseFailed,
                    "failed",
                    SeverityText::Warn,
                    "api request parse failed",
                );
                let fields = event.fields_mut();
                fields.append_json_map(api_base_fields(ctx).into_attributes());
                fields.insert("api.peer_addr", peer.to_string());
                fields.insert("error", message.clone());
                ctx.log
                    .emit_app_event("api_worker::step_once", event)
                    .map_err(|err| {
                        WorkerError::Message(format!("api parse failure log emit failed: {err}"))
                    })?;
                let response = HttpResponse::text(400, "Bad Request", message);
                stream.write_http_response(response).await?;
                return Ok(());
            }
            Err(_elapsed) => return Ok(()),
        };

    match authorize_request(ctx, &cfg, &request)? {
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

    let response = route_request(ctx, &cfg, peer, request).await;
    let status_code = response.status;
    stream.write_http_response(response).await?;

    let mut event = api_event(
        ApiEventKind::ResponseSent,
        "ok",
        SeverityText::Debug,
        "api response sent",
    );
    let fields = event.fields_mut();
    fields.append_json_map(api_base_fields(ctx).into_attributes());
    fields.insert("api.peer_addr", peer.to_string());
    fields.insert("api.status_code", u64::from(status_code));
    ctx.log
        .emit_app_event("api_worker::step_once", event)
        .map_err(|err| WorkerError::Message(format!("api response log emit failed: {err}")))?;
    Ok(())
}

fn api_base_fields(ctx: &ApiWorkerCtx) -> StructuredFields {
    let mut fields = StructuredFields::new();
    fields.insert("scope", ctx.scope.clone());
    fields.insert("member_id", ctx.member_id.clone());
    fields
}

fn extract_request_id(request: &HttpRequest) -> Option<String> {
    request
        .headers
        .iter()
        .find(|(name, _value)| name.eq_ignore_ascii_case("x-request-id"))
        .map(|(_name, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.len() > API_REQUEST_ID_MAX_LEN {
                value[..API_REQUEST_ID_MAX_LEN].to_string()
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
    let mut event = api_event(
        ApiEventKind::AuthDecision,
        "ok",
        SeverityText::Debug,
        "api auth decision",
    );
    let fields = event.fields_mut();
    fields.append_json_map(api_base_fields(ctx).into_attributes());
    fields.insert("api.peer_addr", peer.to_string());
    fields.insert("api.method", request.method.clone());
    fields.insert("api.route_template", route_template(request));
    fields.insert("api.auth.header_present", auth_header_present(request));
    fields.insert("api.auth.result", decision.to_string());
    fields.insert(
        "api.auth.required_role",
        format!("{:?}", endpoint_role(request)).to_lowercase(),
    );
    if let Some(request_id) = extract_request_id(request) {
        fields.insert("api.request_id", request_id);
    }
    ctx.log
        .emit_app_event("api_worker::authorize_request", event)
        .map_err(|err| WorkerError::Message(format!("api auth log emit failed: {err}")))?;
    Ok(())
}

fn is_fatal_api_step_error(err: &WorkerError) -> bool {
    let message = err.to_string();
    message.contains("api accept failed")
        || message.contains("tls mode requires a configured tls acceptor")
        || message.contains("api local_addr failed")
}

async fn route_request(
    ctx: &mut ApiWorkerCtx,
    cfg: &RuntimeConfig,
    _peer: std::net::SocketAddr,
    request: HttpRequest,
) -> HttpResponse {
    let (path, _query) = split_path_and_query(&request.path);
    match (request.method.as_str(), path) {
        ("POST", "/switchover") => {
            let input = match serde_json::from_slice::<SwitchoverRequestInput>(&request.body) {
                Ok(parsed) => parsed,
                Err(err) => {
                    return HttpResponse::text(400, "Bad Request", format!("invalid json: {err}"));
                }
            };
            let Some(dcs_subscriber) = ctx.dcs_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "dcs state unavailable");
            };
            let Some(ha_subscriber) = ctx.ha_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "ha state unavailable");
            };
            let dcs = dcs_subscriber.latest();
            let ha = ha_subscriber.latest();
            match post_switchover(
                &ctx.scope,
                &crate::state::MemberId(ctx.member_id.clone()),
                &ctx.dcs_handle,
                &dcs,
                &ha,
                input,
            )
            .await
            {
                Ok(value) => HttpResponse::json(202, "Accepted", &value),
                Err(err) => api_error_to_http(err),
            }
        }
        ("DELETE", "/switchover") => match delete_switchover(&ctx.scope, &ctx.dcs_handle).await {
            Ok(value) => HttpResponse::json(202, "Accepted", &value),
            Err(err) => api_error_to_http(err),
        },
        ("GET", "/state") => {
            let Some(pg_subscriber) = ctx.pg_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "pg state unavailable");
            };
            let Some(process_subscriber) = ctx.process_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "process state unavailable");
            };
            let Some(dcs_subscriber) = ctx.dcs_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "dcs state unavailable");
            };
            let Some(ha_subscriber) = ctx.ha_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "ha state unavailable");
            };
            let pg = pg_subscriber.latest();
            let process = process_subscriber.latest();
            let dcs = dcs_subscriber.latest();
            let ha = ha_subscriber.latest();
            let response = build_node_state(cfg, &pg, &process, &dcs, &ha);
            HttpResponse::json(200, "OK", &response)
        }
        _ => HttpResponse::text(404, "Not Found", "not found"),
    }
}

fn api_error_to_http(err: ApiError) -> HttpResponse {
    match err {
        ApiError::BadRequest(message) => HttpResponse::text(400, "Bad Request", message),
        ApiError::DcsCommand(message) => HttpResponse::text(503, "Service Unavailable", message),
    }
}

fn split_path_and_query(path: &str) -> (&str, Option<&str>) {
    match path.split_once('?') {
        Some((head, tail)) => (head, Some(tail)),
        None => (path, None),
    }
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
) -> Result<AuthDecision, WorkerError> {
    let tokens = resolve_role_tokens(ctx, cfg)?;
    if tokens.read_token.is_none() && tokens.admin_token.is_none() {
        return Ok(AuthDecision::Allowed);
    }

    let Some(token) = extract_bearer_token(request) else {
        return Ok(AuthDecision::Unauthorized);
    };

    if let Some(expected_admin) = tokens.admin_token.as_deref() {
        if token == expected_admin {
            return Ok(AuthDecision::Allowed);
        }
    }

    Ok(match endpoint_role(request) {
        EndpointRole::Read => {
            if let Some(expected_read) = tokens.read_token.as_deref() {
                if token == expected_read {
                    return Ok(AuthDecision::Allowed);
                }
            }
            AuthDecision::Unauthorized
        }
        EndpointRole::Admin => {
            if let Some(expected_read) = tokens.read_token.as_deref() {
                if token == expected_read {
                    return Ok(AuthDecision::Forbidden);
                }
            }
            AuthDecision::Unauthorized
        }
    })
}

fn resolve_role_tokens(
    ctx: &ApiWorkerCtx,
    cfg: &RuntimeConfig,
) -> Result<ApiRoleTokens, WorkerError> {
    if let Some(configured) = ctx.role_tokens.as_ref() {
        return Ok(configured.clone());
    }

    match &cfg.api.security.auth {
        ApiAuthConfig::Disabled => Ok(ApiRoleTokens {
            read_token: None,
            admin_token: None,
        }),
        ApiAuthConfig::RoleTokens(tokens) => Ok(ApiRoleTokens {
            read_token: resolve_runtime_token(
                "api.security.auth.role_tokens.read_token",
                tokens.read_token.as_ref(),
            )?,
            admin_token: resolve_runtime_token(
                "api.security.auth.role_tokens.admin_token",
                tokens.admin_token.as_ref(),
            )?,
        }),
    }
}

fn endpoint_role(request: &HttpRequest) -> EndpointRole {
    let (path, _query) = split_path_and_query(&request.path);
    match (request.method.as_str(), path) {
        ("POST", "/switchover") | ("DELETE", "/switchover") => EndpointRole::Admin,
        _ => EndpointRole::Read,
    }
}

fn resolve_runtime_token(
    field: &str,
    raw: Option<&crate::config::SecretSource>,
) -> Result<Option<String>, WorkerError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let value = crate::config::resolve_secret_string(field, raw)
        .map_err(|err| WorkerError::Message(err.to_string()))?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
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
                        let mut event = api_event(
                            ApiEventKind::TlsClientCertMissing,
                            "failed",
                            SeverityText::Warn,
                            "tls client cert missing",
                        );
                        let fields = event.fields_mut();
                        fields.append_json_map(api_base_fields(ctx).into_attributes());
                        fields.insert("api.peer_addr", peer.to_string());
                        fields.insert("api.tls_mode", "required");
                        ctx.log
                            .emit_app_event("api_worker::accept_connection", event)
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
                    let mut event = api_event(
                        ApiEventKind::TlsHandshakeFailed,
                        "failed",
                        SeverityText::Warn,
                        "tls handshake failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(api_base_fields(ctx).into_attributes());
                    fields.insert("api.peer_addr", peer.to_string());
                    fields.insert("api.tls_mode", "required");
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("api_worker::accept_connection", event)
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
                        let mut event = api_event(
                            ApiEventKind::TlsClientCertMissing,
                            "failed",
                            SeverityText::Warn,
                            "tls client cert missing",
                        );
                        let fields = event.fields_mut();
                        fields.append_json_map(api_base_fields(ctx).into_attributes());
                        fields.insert("api.peer_addr", peer.to_string());
                        fields.insert("api.tls_mode", "optional");
                        ctx.log
                            .emit_app_event("api_worker::accept_connection", event)
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
                    let mut event = api_event(
                        ApiEventKind::TlsHandshakeFailed,
                        "failed",
                        SeverityText::Warn,
                        "tls handshake failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(api_base_fields(ctx).into_attributes());
                    fields.insert("api.peer_addr", peer.to_string());
                    fields.insert("api.tls_mode", "optional");
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("api_worker::accept_connection", event)
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
    match tokio::time::timeout(API_TLS_CLIENT_HELLO_PEEK_TIMEOUT, stream.peek(&mut first)).await {
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
    let mut buffer = Vec::<u8>::new();
    let mut temp = [0u8; HTTP_REQUEST_SCRATCH_BUFFER_BYTES];
    let mut header_end: Option<usize> = None;
    let mut content_length: Option<usize> = None;

    loop {
        if buffer.len() > HTTP_REQUEST_MAX_BYTES {
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
            } else if buffer.len() > HTTP_REQUEST_HEADER_LIMIT_BYTES {
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
    }; HTTP_REQUEST_HEADER_CAPACITY];
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
