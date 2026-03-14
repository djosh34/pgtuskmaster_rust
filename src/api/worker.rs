use std::net::SocketAddr;

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::trace::TraceLayer;

use crate::{
    api::{
        controller::{
            build_node_state, delete_switchover, post_switchover, NodeStateSnapshot,
            SwitchoverRequest,
        },
        ApiError, NodeState, ReloadCertificatesResponse,
    },
    config::{ApiAuthConfig, RuntimeConfig},
    dcs::{DcsHandle, DcsView},
    ha::state::HaState,
    logging::LogHandle,
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{StateSubscriber, WorkerError},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ApiClusterIdentity {
    pub(crate) cluster_name: String,
    pub(crate) scope: String,
    pub(crate) member_id: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(test, allow(dead_code))]
pub(crate) enum ApiObservedState {
    Unavailable,
    Live {
        pg: StateSubscriber<PgInfoState>,
        process: StateSubscriber<ProcessState>,
        dcs: StateSubscriber<DcsView>,
        ha: StateSubscriber<HaState>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ResolvedApiRoleTokens {
    read_token: Option<String>,
    admin_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ApiAuthState {
    Disabled,
    RoleTokens(ResolvedApiRoleTokens),
}

#[derive(Clone, Debug)]
pub(crate) enum ApiBindConfig {
    Listen(SocketAddr),
}

impl ApiBindConfig {
    pub(crate) fn listen(listen_addr: SocketAddr) -> Self {
        Self::Listen(listen_addr)
    }
}

#[derive(Clone)]
pub(crate) struct ApiTlsRuntime {
    pub(crate) server_config: RustlsConfig,
}

#[derive(Clone)]
pub(crate) enum ApiServerTransport {
    Http,
    Https(ApiTlsRuntime),
}

#[derive(Clone)]
pub(crate) enum ApiCertificateReloadHandle {
    Disabled,
    Https { server_config: RustlsConfig },
}

impl ApiCertificateReloadHandle {
    pub(crate) fn from_transport(transport: &ApiServerTransport) -> Self {
        match transport {
            ApiServerTransport::Http => Self::Disabled,
            ApiServerTransport::Https(runtime) => Self::Https {
                server_config: runtime.server_config.clone(),
            },
        }
    }

    async fn reload(&self, cfg: &RuntimeConfig) -> Result<bool, WorkerError> {
        match self {
            Self::Disabled => Ok(false),
            Self::Https { server_config } => match &cfg.api.transport {
                crate::config::ApiTransportConfig::Http => Err(WorkerError::Message(
                    "api cert reload requires https transport".to_string(),
                )),
                crate::config::ApiTransportConfig::Https { tls } => {
                    let reloaded = crate::tls::build_api_server_config(tls)
                        .map_err(|err| WorkerError::Message(err.to_string()))?;
                    server_config.reload_from_config(reloaded);
                    Ok(true)
                }
            },
        }
    }
}

pub struct ApiServerCtx {
    pub(crate) identity: ApiClusterIdentity,
    pub(crate) observed: ApiObservedState,
    pub(crate) control: ApiControlPlane,
    pub(crate) serving: ApiServingPlan,
    pub(crate) log: LogHandle,
}

#[derive(Clone)]
pub(crate) struct ApiControlPlane {
    pub(crate) runtime_config: StateSubscriber<RuntimeConfig>,
    pub(crate) dcs_handle: DcsHandle,
}

#[derive(Clone)]
pub(crate) struct ApiServingPlan {
    pub(crate) bind: ApiBindConfig,
    pub(crate) auth: ApiAuthState,
    pub(crate) transport: ApiServerTransport,
    pub(crate) cert_reloader: ApiCertificateReloadHandle,
}

#[derive(Clone)]
struct ApiAppState {
    identity: ApiClusterIdentity,
    runtime_config: StateSubscriber<RuntimeConfig>,
    dcs_handle: DcsHandle,
    state: ApiObservedState,
    auth: ApiAuthState,
    cert_reloader: ApiCertificateReloadHandle,
    _log: LogHandle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RequiredRole {
    Read,
    Admin,
}

#[derive(Clone, Debug)]
struct ApiHttpError {
    status: StatusCode,
    message: String,
}

impl ApiHttpError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, message)
    }

    fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for ApiHttpError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

impl From<ApiError> for ApiHttpError {
    fn from(value: ApiError) -> Self {
        match value {
            ApiError::BadRequest(message) => Self::new(StatusCode::BAD_REQUEST, message),
            ApiError::DcsCommand(message) => Self::new(StatusCode::SERVICE_UNAVAILABLE, message),
        }
    }
}

pub(crate) fn build_router(ctx: ApiServerCtx) -> Result<Router, WorkerError> {
    let (_bind, _transport, app_state) = build_app_state(ctx)?;
    Ok(router_from_state(app_state))
}

fn build_app_state(
    ctx: ApiServerCtx,
) -> Result<(ApiBindConfig, ApiServerTransport, ApiAppState), WorkerError> {
    let ApiServerCtx {
        identity,
        observed,
        control,
        serving,
        log,
    } = ctx;
    let ApiControlPlane {
        runtime_config,
        dcs_handle,
    } = control;
    let ApiServingPlan {
        bind,
        auth,
        transport,
        cert_reloader,
    } = serving;
    let auth = resolve_auth_state(&auth, &runtime_config.latest())?;
    let app_state = ApiAppState {
        identity,
        runtime_config,
        dcs_handle,
        state: observed,
        auth,
        cert_reloader,
        _log: log,
    };
    Ok((bind, transport, app_state))
}

fn router_from_state(app_state: ApiAppState) -> Router {
    let read_routes =
        Router::new()
            .route("/state", get(get_state))
            .route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                require_read_auth,
            ));
    let admin_routes = Router::new()
        .route("/switchover", post(post_switchover_handler))
        .route("/switchover", delete(delete_switchover_handler))
        .route("/reload/certs", post(reload_certificates))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_admin_auth,
        ));

    Router::new()
        .merge(read_routes)
        .merge(admin_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

pub async fn run(ctx: ApiServerCtx) -> Result<(), WorkerError> {
    let (bind, transport, app_state) = build_app_state(ctx)?;
    let app = router_from_state(app_state);

    match (bind, transport) {
        (ApiBindConfig::Listen(listen_addr), ApiServerTransport::Http) => {
            axum_server::bind(listen_addr)
                .serve(app.into_make_service())
                .await
                .map_err(|err| WorkerError::Message(format!("api server failed: {err}")))
        }
        (ApiBindConfig::Listen(listen_addr), ApiServerTransport::Https(runtime)) => {
            axum_server::bind_rustls(listen_addr, runtime.server_config)
                .serve(app.into_make_service())
                .await
                .map_err(|err| WorkerError::Message(format!("api server failed: {err}")))
        }
    }
}

async fn get_state(State(state): State<ApiAppState>) -> Result<Json<NodeState>, ApiHttpError> {
    let ApiObservedState::Live {
        pg,
        process,
        dcs,
        ha,
    } = &state.state
    else {
        return Err(ApiHttpError::service_unavailable(
            "state subscribers unavailable",
        ));
    };
    let runtime_config = state.runtime_config.latest();
    let snapshot = NodeStateSnapshot {
        cluster_name: state.identity.cluster_name.clone(),
        scope: state.identity.scope.clone(),
        self_member_id: state.identity.member_id.clone(),
        pg: pg.latest(),
        process: process.latest(),
        dcs: dcs.latest(),
        ha: ha.latest(),
    };
    Ok(Json(build_node_state(&runtime_config, snapshot)))
}

async fn post_switchover_handler(
    State(state): State<ApiAppState>,
    Json(request): Json<SwitchoverRequest>,
) -> Result<(StatusCode, Json<crate::api::AcceptedResponse>), ApiHttpError> {
    let ApiObservedState::Live { dcs, ha, .. } = &state.state else {
        return Err(ApiHttpError::service_unavailable(
            "state subscribers unavailable",
        ));
    };
    let response = post_switchover(
        state.identity.scope.as_str(),
        &crate::state::MemberId(state.identity.member_id.clone()),
        &state.dcs_handle,
        &dcs.latest(),
        &ha.latest(),
        request,
    )
    .await?;
    Ok((StatusCode::ACCEPTED, Json(response)))
}

async fn delete_switchover_handler(
    State(state): State<ApiAppState>,
) -> Result<(StatusCode, Json<crate::api::AcceptedResponse>), ApiHttpError> {
    let response = delete_switchover(state.identity.scope.as_str(), &state.dcs_handle).await?;
    Ok((StatusCode::ACCEPTED, Json(response)))
}

async fn reload_certificates(
    State(state): State<ApiAppState>,
) -> Result<Json<ReloadCertificatesResponse>, ApiHttpError> {
    let reloaded = state
        .cert_reloader
        .reload(&state.runtime_config.latest())
        .await
        .map_err(|err| ApiHttpError::internal(err.to_string()))?;
    Ok(Json(ReloadCertificatesResponse { reloaded }))
}

async fn require_read_auth(
    State(state): State<ApiAppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiHttpError> {
    require_auth(state, RequiredRole::Read, request, next).await
}

async fn require_admin_auth(
    State(state): State<ApiAppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiHttpError> {
    require_auth(state, RequiredRole::Admin, request, next).await
}

async fn require_auth(
    state: ApiAppState,
    required_role: RequiredRole,
    request: Request,
    next: Next,
) -> Result<Response, ApiHttpError> {
    match authorize_request(&state.auth, required_role, &request) {
        AuthDecision::Allowed => Ok(next.run(request).await),
        AuthDecision::Unauthorized => {
            Err(ApiHttpError::new(StatusCode::UNAUTHORIZED, "unauthorized"))
        }
        AuthDecision::Forbidden => Err(ApiHttpError::new(StatusCode::FORBIDDEN, "forbidden")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthDecision {
    Allowed,
    Unauthorized,
    Forbidden,
}

fn authorize_request(
    auth: &ApiAuthState,
    required_role: RequiredRole,
    request: &Request,
) -> AuthDecision {
    let ApiAuthState::RoleTokens(tokens) = auth else {
        return AuthDecision::Allowed;
    };

    if tokens.read_token.is_none() && tokens.admin_token.is_none() {
        return AuthDecision::Allowed;
    }

    let Some(token) = extract_bearer_token(request) else {
        return AuthDecision::Unauthorized;
    };

    if tokens
        .admin_token
        .as_deref()
        .is_some_and(|expected| expected == token)
    {
        return AuthDecision::Allowed;
    }

    match required_role {
        RequiredRole::Read => {
            if tokens
                .read_token
                .as_deref()
                .is_some_and(|expected| expected == token)
            {
                AuthDecision::Allowed
            } else {
                AuthDecision::Unauthorized
            }
        }
        RequiredRole::Admin => {
            if tokens
                .read_token
                .as_deref()
                .is_some_and(|expected| expected == token)
            {
                AuthDecision::Forbidden
            } else {
                AuthDecision::Unauthorized
            }
        }
    }
}

fn extract_bearer_token(request: &Request) -> Option<&str> {
    let header = request.headers().get(AUTHORIZATION)?.to_str().ok()?.trim();
    header.strip_prefix("Bearer ").map(str::trim)
}

fn resolve_auth_state(
    configured: &ApiAuthState,
    cfg: &RuntimeConfig,
) -> Result<ApiAuthState, WorkerError> {
    match configured {
        ApiAuthState::Disabled => match &cfg.api.auth {
            ApiAuthConfig::Disabled => Ok(ApiAuthState::Disabled),
            ApiAuthConfig::RoleTokens(tokens) => {
                Ok(ApiAuthState::RoleTokens(ResolvedApiRoleTokens {
                    read_token: resolve_runtime_token(
                        "api.security.auth.role_tokens.read_token",
                        tokens.read_token.as_ref(),
                    )?,
                    admin_token: resolve_runtime_token(
                        "api.security.auth.role_tokens.admin_token",
                        tokens.admin_token.as_ref(),
                    )?,
                }))
            }
        },
        ApiAuthState::RoleTokens(tokens) => Ok(ApiAuthState::RoleTokens(tokens.clone())),
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

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use tower::util::ServiceExt;

    use crate::{
        config::{
            ApiAuthConfig, ApiClientAuthConfig, ApiRoleTokensConfig, ApiTlsConfig,
            ApiTransportConfig, InlineOrPath, SecretSource, TlsServerIdentityConfig,
        },
        dcs::DcsHandle,
        dev_support::{runtime_config::RuntimeConfigBuilder, tls::build_adversarial_tls_fixture},
        logging::LogHandle,
        state::new_state_channel,
    };

    use super::{
        build_router, ApiAuthState, ApiBindConfig, ApiCertificateReloadHandle, ApiClusterIdentity,
        ApiControlPlane, ApiObservedState, ApiServerCtx, ApiServingPlan,
    };

    fn sample_admin_request(uri: &str) -> Result<Request<Body>, String> {
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("authorization", "Bearer admin-secret")
            .body(Body::empty())
            .map_err(|err| err.to_string())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reload_certificates_succeeds_for_https_transport() -> Result<(), String> {
        let fixture = build_adversarial_tls_fixture().map_err(|err| err.to_string())?;
        let cfg = RuntimeConfigBuilder::new()
            .transform_api(|api| crate::config::ApiConfig {
                transport: ApiTransportConfig::Https {
                    tls: ApiTlsConfig {
                        identity: TlsServerIdentityConfig {
                            cert_chain: InlineOrPath::Inline {
                                content: fixture.valid_server.cert_pem.clone(),
                            },
                            private_key: InlineOrPath::Inline {
                                content: fixture.valid_server.key_pem.clone(),
                            },
                        },
                        client_auth: ApiClientAuthConfig::Disabled,
                    },
                },
                auth: ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
                    read_token: Some(SecretSource::Inline {
                        content: "read-secret".to_string(),
                    }),
                    admin_token: Some(SecretSource::Inline {
                        content: "admin-secret".to_string(),
                    }),
                }),
                ..api
            })
            .build();

        let (_cfg_publisher, runtime_config) = new_state_channel(cfg.clone());
        let (_pg_publisher, pg) = new_state_channel(crate::pginfo::state::PgInfoState::starting());
        let (_process_publisher, process) =
            new_state_channel(crate::process::state::ProcessState::starting());
        let (_dcs_publisher, dcs) = new_state_channel(crate::dcs::DcsView::starting());
        let (_ha_publisher, ha) = new_state_channel(crate::ha::state::HaState::initial(
            crate::state::WorkerStatus::Starting,
        ));
        let transport = crate::tls::build_api_server_transport(&cfg.api.transport)
            .map_err(|err| err.to_string())?;
        let app = build_router(ApiServerCtx {
            identity: ApiClusterIdentity {
                cluster_name: cfg.cluster.name.clone(),
                scope: cfg.cluster.scope.clone(),
                member_id: cfg.cluster.member_id.clone(),
            },
            observed: ApiObservedState::Live {
                pg,
                process,
                dcs,
                ha,
            },
            control: ApiControlPlane {
                runtime_config,
                dcs_handle: DcsHandle::closed(),
            },
            serving: ApiServingPlan {
                bind: ApiBindConfig::listen(cfg.api.listen_addr),
                auth: ApiAuthState::Disabled,
                transport: transport.clone(),
                cert_reloader: ApiCertificateReloadHandle::from_transport(&transport),
            },
            log: LogHandle::disabled(),
        })
        .map_err(|err| err.to_string())?;

        let response = app
            .oneshot(sample_admin_request("/reload/certs")?)
            .await
            .map_err(|err| err.to_string())?;
        if response.status() != StatusCode::OK {
            return Err(format!("unexpected status {}", response.status()));
        }
        Ok(())
    }
}
