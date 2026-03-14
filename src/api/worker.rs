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
        ApiCertificateReloadStep, ApiError, NodeState, PostgresCertificateReloadStep,
        PostgresReloadSignal, ReloadCertificatesResponse,
    },
    config::{ApiAuthConfig, RuntimeConfig},
    dcs::{DcsHandle, DcsView},
    ha::state::HaState,
    logging::LogHandle,
    pginfo::state::PgInfoState,
    process::postmaster::{reload_managed_postmaster, ManagedPostmasterTarget},
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
pub(crate) enum ApiTlsCertificateReloadHandle {
    HttpTransport,
    Https { server_config: RustlsConfig },
}

impl ApiTlsCertificateReloadHandle {
    pub(crate) fn from_transport(transport: &ApiServerTransport) -> Self {
        match transport {
            ApiServerTransport::Http => Self::HttpTransport,
            ApiServerTransport::Https(runtime) => Self::Https {
                server_config: runtime.server_config.clone(),
            },
        }
    }

    async fn reload(
        &self,
        cfg: &RuntimeConfig,
    ) -> Result<ApiCertificateReloadStep, ReloadCertificatesError> {
        match self {
            Self::HttpTransport => Ok(ApiCertificateReloadStep::HttpTransportUnchanged),
            Self::Https { server_config } => match &cfg.api.transport {
                crate::config::ApiTransportConfig::Http => Err(ReloadCertificatesError::Api {
                    message: "api cert reload requires https transport".to_string(),
                }),
                crate::config::ApiTransportConfig::Https { tls } => {
                    let reloaded = crate::tls::build_api_server_config(tls).map_err(|err| {
                        ReloadCertificatesError::Api {
                            message: err.to_string(),
                        }
                    })?;
                    server_config.reload_from_config(reloaded);
                    Ok(ApiCertificateReloadStep::HttpsConfigurationReloaded)
                }
            },
        }
    }
}

#[derive(Clone)]
pub(crate) struct ApiReloadCertificatesHandle {
    api_tls: ApiTlsCertificateReloadHandle,
}

impl ApiReloadCertificatesHandle {
    pub(crate) fn from_transport(transport: &ApiServerTransport) -> Self {
        Self {
            api_tls: ApiTlsCertificateReloadHandle::from_transport(transport),
        }
    }

    async fn reload(
        &self,
        cfg: &RuntimeConfig,
    ) -> Result<ReloadCertificatesResponse, ReloadCertificatesError> {
        let api = self.api_tls.reload(cfg).await?;
        let target = ManagedPostmasterTarget::from_data_dir(cfg.postgres.paths.data_dir.clone());
        let postgres = reload_managed_postmaster(&target)?;
        Ok(ReloadCertificatesResponse {
            api,
            postgres: PostgresCertificateReloadStep {
                signal: PostgresReloadSignal::Sighup,
                postmaster_pid: postgres.postmaster.pid.value(),
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
enum ReloadCertificatesError {
    #[error("api certificate reload failed: {message}")]
    Api { message: String },
    #[error("postgres certificate reload failed: {0}")]
    Postgres(#[from] crate::process::postmaster::ManagedPostmasterError),
}

pub(crate) struct ApiServerCtx {
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
    pub(crate) reload_certificates: ApiReloadCertificatesHandle,
}

#[derive(Clone)]
struct ApiAppState {
    identity: ApiClusterIdentity,
    runtime_config: StateSubscriber<RuntimeConfig>,
    dcs_handle: DcsHandle,
    state: ApiObservedState,
    auth: ApiAuthState,
    reload_certificates: ApiReloadCertificatesHandle,
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
        reload_certificates,
    } = serving;
    let auth = resolve_auth_state(&auth, &runtime_config.latest())?;
    let app_state = ApiAppState {
        identity,
        runtime_config,
        dcs_handle,
        state: observed,
        auth,
        reload_certificates,
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

pub(crate) async fn run(ctx: ApiServerCtx) -> Result<(), WorkerError> {
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
        .reload_certificates
        .reload(&state.runtime_config.latest())
        .await
        .map_err(|err| ApiHttpError::internal(err.to_string()))?;
    Ok(Json(reloaded))
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
    use std::{
        fs,
        path::{Path, PathBuf},
        process::{Child, Command},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use axum::{
        body::{to_bytes, Body},
        http::{Method, Request, StatusCode},
        Router,
    };
    use tower::util::ServiceExt;

    use crate::{
        api::{ApiCertificateReloadStep, PostgresReloadSignal, ReloadCertificatesResponse},
        config::{
            ApiAuthConfig, ApiClientAuthConfig, ApiRoleTokensConfig, ApiTlsConfig,
            ApiTransportConfig, InlineOrPath, RuntimeConfig, SecretSource, TlsServerIdentityConfig,
        },
        dcs::DcsHandle,
        dev_support::{runtime_config::RuntimeConfigBuilder, tls::build_adversarial_tls_fixture},
        logging::LogHandle,
        process::postmaster::{lookup_managed_postmaster, ManagedPostmasterTarget},
        state::{new_state_channel, StatePublisher},
    };

    use super::{
        build_router, ApiAuthState, ApiBindConfig, ApiClusterIdentity, ApiControlPlane,
        ApiObservedState, ApiReloadCertificatesHandle, ApiServerCtx, ApiServingPlan,
    };

    struct ChildGuard(Option<Child>);

    impl ChildGuard {
        fn child(&self) -> Result<&Child, String> {
            self.0
                .as_ref()
                .ok_or_else(|| "fake postgres child handle missing".to_string())
        }

        fn child_mut(&mut self) -> Result<&mut Child, String> {
            self.0
                .as_mut()
                .ok_or_else(|| "fake postgres child handle missing".to_string())
        }
    }

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            if let Some(child) = self.0.as_mut() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    fn sample_admin_request(uri: &str) -> Result<Request<Body>, String> {
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("authorization", "Bearer admin-secret")
            .body(Body::empty())
            .map_err(|err| err.to_string())
    }

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error for test dir: {err}"))?
            .as_millis();
        let dir = std::env::temp_dir().join(format!(
            "pgtm-api-worker-{label}-{}-{millis}",
            std::process::id()
        ));
        fs::create_dir_all(&dir)
            .map_err(|err| format!("create test dir {} failed: {err}", dir.display()))?;
        Ok(dir)
    }

    fn sample_https_runtime_config(data_dir: PathBuf) -> Result<RuntimeConfig, String> {
        let fixture = build_adversarial_tls_fixture().map_err(|err| err.to_string())?;
        Ok(RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
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
            .build())
    }

    fn sample_invalid_https_runtime_config(data_dir: PathBuf) -> RuntimeConfig {
        RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .transform_api(|api| crate::config::ApiConfig {
                transport: ApiTransportConfig::Https {
                    tls: ApiTlsConfig {
                        identity: TlsServerIdentityConfig {
                            cert_chain: InlineOrPath::Inline {
                                content: "not a certificate".to_string(),
                            },
                            private_key: InlineOrPath::Inline {
                                content: "not a key".to_string(),
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
            .build()
    }

    fn sample_http_runtime_config(data_dir: PathBuf) -> RuntimeConfig {
        RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .transform_api(|api| crate::config::ApiConfig {
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
            .build()
    }

    fn build_test_app(
        cfg: RuntimeConfig,
    ) -> Result<(Router, StatePublisher<RuntimeConfig>), String> {
        let (cfg_publisher, runtime_config) = new_state_channel(cfg.clone());
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
                reload_certificates: ApiReloadCertificatesHandle::from_transport(&transport),
            },
            log: LogHandle::disabled(),
        })
        .map_err(|err| err.to_string())?;
        Ok((app, cfg_publisher))
    }

    async fn response_body_text(response: axum::response::Response) -> Result<String, String> {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .map_err(|err| err.to_string())?;
        String::from_utf8(bytes.to_vec()).map_err(|err| err.to_string())
    }

    async fn response_body_json(
        response: axum::response::Response,
    ) -> Result<ReloadCertificatesResponse, String> {
        let body = response_body_text(response).await?;
        serde_json::from_str::<ReloadCertificatesResponse>(&body).map_err(|err| err.to_string())
    }

    #[cfg(unix)]
    fn spawn_fake_postgres_process(
        root: &Path,
        data_dir: &Path,
        signal_log: &Path,
    ) -> Result<ChildGuard, String> {
        let script = root.join("fake-postgres.sh");
        let script_contents = format!(
            "#!/bin/bash\ntrap 'printf hup >> \"{}\"' HUP\nwhile true; do read -r -t 1 _ || true; done\n",
            signal_log.display()
        );
        fs::write(&script, script_contents).map_err(|err| {
            format!(
                "write fake postgres script {} failed: {err}",
                script.display()
            )
        })?;
        let mut permissions = fs::metadata(&script)
            .map_err(|err| {
                format!(
                    "read fake postgres script metadata {} failed: {err}",
                    script.display()
                )
            })?
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&script, permissions).map_err(|err| {
            format!(
                "set fake postgres script permissions {} failed: {err}",
                script.display()
            )
        })?;
        let child = Command::new("/bin/bash")
            .arg("-lc")
            .arg(format!(
                "exec -a postgres /bin/bash '{}' '{}'",
                script.display(),
                data_dir.display()
            ))
            .spawn()
            .map_err(|err| {
                format!(
                    "spawn fake postgres process via {} failed: {err}",
                    script.display()
                )
            })?;
        Ok(ChildGuard(Some(child)))
    }

    fn write_postmaster_pid(
        data_dir: &Path,
        pid: u32,
        recorded_data_dir: &Path,
    ) -> Result<(), String> {
        let pid_file = data_dir.join("postmaster.pid");
        let contents = format!("{pid}\n{}\n", recorded_data_dir.display());
        fs::write(&pid_file, contents).map_err(|err| {
            format!(
                "write postmaster pid file {} failed: {err}",
                pid_file.display()
            )
        })
    }

    fn wait_for_signal_log(signal_log: &Path) -> Result<String, String> {
        let mut attempts = 0_u8;
        while attempts < 150 {
            match fs::read_to_string(signal_log) {
                Ok(contents) if !contents.is_empty() => return Ok(contents),
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(format!(
                        "read signal log {} failed: {err}",
                        signal_log.display()
                    ));
                }
            }
            std::thread::sleep(Duration::from_millis(10));
            attempts = attempts.saturating_add(1);
        }
        Err(format!(
            "signal log {} was not written in time",
            signal_log.display()
        ))
    }

    fn wait_for_managed_postmaster_ready(data_dir: &Path) -> Result<(), String> {
        let target = ManagedPostmasterTarget::from_data_dir(data_dir.to_path_buf());
        let mut attempts = 0_u8;
        while attempts < 50 {
            if lookup_managed_postmaster(&target).is_ok() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
            attempts = attempts.saturating_add(1);
        }
        Err(format!(
            "managed postmaster never became ready for {}",
            data_dir.display()
        ))
    }

    async fn assert_no_signal_written(signal_log: &Path) -> Result<(), String> {
        tokio::time::sleep(Duration::from_millis(50)).await;
        match fs::read_to_string(signal_log) {
            Ok(contents) if contents.is_empty() => Ok(()),
            Ok(contents) => Err(format!(
                "signal log {} should be empty but contained {contents:?}",
                signal_log.display()
            )),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(format!(
                "read signal log {} failed: {err}",
                signal_log.display()
            )),
        }
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn reload_certificates_succeeds_for_https_transport_and_signals_postgres(
    ) -> Result<(), String> {
        let root = unique_test_dir("reload-success")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let child = spawn_fake_postgres_process(&root, &data_dir, &signal_log)?;
        let pid = child.child()?.id();
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let _child = child;
        wait_for_managed_postmaster_ready(&data_dir)?;
        let cfg = sample_https_runtime_config(data_dir)?;
        let (app, _cfg_publisher) = build_test_app(cfg)?;

        let response = app
            .oneshot(sample_admin_request("/reload/certs")?)
            .await
            .map_err(|err| err.to_string())?;
        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = response_body_text(response).await?;
            return Err(format!("unexpected status {status}: {body}"));
        }
        let body = response_body_json(response).await?;
        if body.api != ApiCertificateReloadStep::HttpsConfigurationReloaded {
            return Err(format!("unexpected api reload step: {:?}", body.api));
        }
        if body.postgres.signal != PostgresReloadSignal::Sighup {
            return Err(format!(
                "unexpected postgres reload signal: {:?}",
                body.postgres.signal
            ));
        }
        if body.postgres.postmaster_pid != pid {
            return Err(format!(
                "unexpected reloaded pid: expected={pid} actual={}",
                body.postgres.postmaster_pid
            ));
        }
        let contents = wait_for_signal_log(&signal_log)?;
        if !contents.contains("hup") {
            return Err(format!(
                "signal log {} did not record hup: {contents:?}",
                signal_log.display()
            ));
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reload_certificates_returns_error_when_postmaster_pid_is_missing() -> Result<(), String>
    {
        let root = unique_test_dir("reload-missing-postmaster")?;
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let cfg = sample_http_runtime_config(data_dir);
        let (app, _cfg_publisher) = build_test_app(cfg)?;

        let response = app
            .oneshot(sample_admin_request("/reload/certs")?)
            .await
            .map_err(|err| err.to_string())?;

        if response.status() != StatusCode::INTERNAL_SERVER_ERROR {
            return Err(format!("unexpected status {}", response.status()));
        }
        let body = response_body_text(response).await?;
        if !body.contains("postgres certificate reload failed") {
            return Err(format!(
                "response body did not mention postgres failure: {body}"
            ));
        }
        if !body.contains("postmaster pid file") {
            return Err(format!(
                "response body did not mention missing pid file: {body}"
            ));
        }
        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn reload_certificates_returns_error_when_postmaster_pid_is_stale() -> Result<(), String>
    {
        let root = unique_test_dir("reload-stale-postmaster")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let mut child = spawn_fake_postgres_process(&root, &data_dir, &signal_log)?;
        let pid = child.child()?.id();
        child
            .child_mut()?
            .kill()
            .map_err(|err| format!("kill fake postgres pid={pid} failed: {err}"))?;
        child
            .child_mut()?
            .wait()
            .map_err(|err| format!("wait fake postgres pid={pid} failed: {err}"))?;
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let cfg = sample_http_runtime_config(data_dir);
        let (app, _cfg_publisher) = build_test_app(cfg)?;

        let response = app
            .oneshot(sample_admin_request("/reload/certs")?)
            .await
            .map_err(|err| err.to_string())?;

        if response.status() != StatusCode::INTERNAL_SERVER_ERROR {
            return Err(format!("unexpected status {}", response.status()));
        }
        let body = response_body_text(response).await?;
        if !body.contains("is not running") {
            return Err(format!("response body did not mention stale pid: {body}"));
        }
        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn reload_certificates_returns_error_when_postmaster_data_dir_mismatches(
    ) -> Result<(), String> {
        let root = unique_test_dir("reload-mismatch")?;
        let target_data_dir = root.join("target-data");
        let real_data_dir = root.join("real-data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&target_data_dir).map_err(|err| {
            format!(
                "create target data dir {} failed: {err}",
                target_data_dir.display()
            )
        })?;
        fs::create_dir_all(&real_data_dir).map_err(|err| {
            format!(
                "create real data dir {} failed: {err}",
                real_data_dir.display()
            )
        })?;
        let child = spawn_fake_postgres_process(&root, &real_data_dir, &signal_log)?;
        let pid = child.child()?.id();
        write_postmaster_pid(&target_data_dir, pid, &real_data_dir)?;
        let _child = child;
        let cfg = sample_http_runtime_config(target_data_dir.clone());
        let (app, _cfg_publisher) = build_test_app(cfg)?;

        let response = app
            .oneshot(sample_admin_request("/reload/certs")?)
            .await
            .map_err(|err| err.to_string())?;

        if response.status() != StatusCode::INTERNAL_SERVER_ERROR {
            return Err(format!("unexpected status {}", response.status()));
        }
        let body = response_body_text(response).await?;
        if !body.contains("does not match managed data dir") {
            return Err(format!(
                "response body did not mention data dir mismatch: {body}"
            ));
        }
        if !body.contains(target_data_dir.display().to_string().as_str()) {
            return Err(format!(
                "response body did not include expected data dir {}: {body}",
                target_data_dir.display()
            ));
        }
        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn reload_certificates_does_not_signal_postgres_when_api_reload_fails(
    ) -> Result<(), String> {
        let root = unique_test_dir("reload-ordering")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let child = spawn_fake_postgres_process(&root, &data_dir, &signal_log)?;
        let pid = child.child()?.id();
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let _child = child;
        wait_for_managed_postmaster_ready(&data_dir)?;
        let cfg = sample_https_runtime_config(data_dir.clone())?;
        let (app, cfg_publisher) = build_test_app(cfg)?;
        cfg_publisher
            .publish(sample_invalid_https_runtime_config(data_dir))
            .map_err(|err| err.to_string())?;

        let response = app
            .oneshot(sample_admin_request("/reload/certs")?)
            .await
            .map_err(|err| err.to_string())?;

        if response.status() != StatusCode::INTERNAL_SERVER_ERROR {
            return Err(format!("unexpected status {}", response.status()));
        }
        let body = response_body_text(response).await?;
        if !body.contains("api certificate reload failed") {
            return Err(format!(
                "response body did not mention api reload failure: {body}"
            ));
        }
        assert_no_signal_written(&signal_log).await?;
        Ok(())
    }
}
