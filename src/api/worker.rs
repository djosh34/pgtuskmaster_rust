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
        controller::{delete_switchover, post_switchover, SwitchoverRequest},
        ApiCertificateReloadStep, ApiError, NodeState, PostgresCertificateReloadStep,
        PostgresReloadSignal, ReloadCertificatesResponse,
    },
    config_v2::{types::ApiAuth, RuntimeConfigV2},
    dcs::{DcsHandle, DcsSnapshot},
    ha::state::HaState,
    logging::LogSender,
    pginfo::state::PgInfoState,
    process::postmaster::{reload_managed_postmaster, ManagedPostmasterTarget},
    process::state::ProcessState,
    state::{StateSubscriber, WorkerError},
};

#[derive(Clone, Debug)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum ApiObservedState {
    Unavailable,
    Live {
        pg: StateSubscriber<PgInfoState>,
        process: StateSubscriber<ProcessState>,
        dcs: StateSubscriber<DcsSnapshot>,
        ha: StateSubscriber<HaState>,
    },
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
        cfg: &RuntimeConfigV2,
    ) -> Result<ApiCertificateReloadStep, ReloadCertificatesError> {
        match self {
            Self::HttpTransport => Ok(ApiCertificateReloadStep::HttpTransportUnchanged),
            Self::Https { server_config } => {
                let crate::config_v2::types::ApiTransport::Https { .. } = &cfg.api.transport else {
                    return Err(ReloadCertificatesError::ApiTransportMismatch);
                };
                let reloaded = crate::tls::build_api_server_config_v2(&cfg.api.transport)?;
                server_config.reload_from_config(reloaded);
                Ok(ApiCertificateReloadStep::HttpsConfigurationReloaded)
            }
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
        cfg: &RuntimeConfigV2,
    ) -> Result<ReloadCertificatesResponse, ReloadCertificatesError> {
        let api = self.api_tls.reload(cfg).await?;
        let target = ManagedPostmasterTarget::from_data_dir(cfg.postgres.data_dir.clone());
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
    #[error("api certificate reload requires https transport")]
    ApiTransportMismatch,
    #[error("api certificate reload failed: {0}")]
    ApiTls(#[from] crate::tls::TlsConfigError),
    #[error("postgres certificate reload failed: {0}")]
    Postgres(#[from] crate::process::postmaster::ManagedPostmasterError),
}

#[derive(Clone)]
pub(crate) struct ApiRuntimeCtx<'a> {
    pub(crate) cfg: &'a RuntimeConfigV2,
    pub(crate) observed: ApiObservedState,
    pub(crate) dcs_handle: DcsHandle,
    pub(crate) transport: ApiServerTransport,
    pub(crate) reload_certificates: ApiReloadCertificatesHandle,
    pub(crate) _log: LogSender,
}

impl<'a> ApiRuntimeCtx<'a> {
    pub(crate) fn new(
        cfg: &'a RuntimeConfigV2,
        dcs_handle: DcsHandle,
        observed: ApiObservedState,
        log: LogSender,
    ) -> Result<Self, WorkerError> {
        let transport = crate::tls::build_api_server_transport_v2(&cfg.api.transport)
            .map_err(|err| WorkerError::Message(format!("api tls config build failed: {err}")))?;

        Ok(Self {
            cfg,
            observed,
            dcs_handle,
            reload_certificates: ApiReloadCertificatesHandle::from_transport(&transport),
            transport,
            _log: log,
        })
    }
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

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_router(ctx: ApiRuntimeCtx<'static>) -> Result<Router, WorkerError> {
    let (_bind, _transport, app_state) = build_app_state(ctx)?;
    Ok(router_from_state(app_state))
}

fn build_app_state(
    ctx: ApiRuntimeCtx<'static>,
) -> Result<(SocketAddr, ApiServerTransport, ApiRuntimeCtx<'static>), WorkerError> {
    let bind = ctx.cfg.api.listen_addr;
    let transport = ctx.transport.clone();
    Ok((bind, transport, ctx))
}

fn router_from_state(app_state: ApiRuntimeCtx<'static>) -> Router {
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

pub(crate) async fn run(ctx: ApiRuntimeCtx<'static>) -> Result<(), WorkerError> {
    let (listen_addr, transport, app_state) = build_app_state(ctx)?;
    let app = router_from_state(app_state);

    match transport {
        ApiServerTransport::Http => axum_server::bind(listen_addr)
            .serve(app.into_make_service())
            .await
            .map_err(|err| WorkerError::Message(format!("api server failed: {err}"))),
        ApiServerTransport::Https(runtime) => {
            axum_server::bind_rustls(listen_addr, runtime.server_config)
                .serve(app.into_make_service())
                .await
                .map_err(|err| WorkerError::Message(format!("api server failed: {err}")))
        }
    }
}

async fn get_state(
    State(state): State<ApiRuntimeCtx<'_>>,
) -> Result<Json<NodeState>, ApiHttpError> {
    let ApiObservedState::Live {
        pg,
        process,
        dcs,
        ha,
    } = &state.observed
    else {
        return Err(ApiHttpError::service_unavailable(
            "state subscribers unavailable",
        ));
    };
    Ok(Json(NodeState {
        identity: crate::state::NodeIdentity {
            cluster_name: state.cfg.cluster_name.clone(),
            scope: state.cfg.scope.clone(),
            member_id: state.cfg.member_id.clone(),
        },
        pg: pg.latest(),
        process: process.latest(),
        dcs: dcs.latest(),
        ha: ha.latest(),
    }))
}

async fn post_switchover_handler(
    State(state): State<ApiRuntimeCtx<'_>>,
    Json(request): Json<SwitchoverRequest>,
) -> Result<(StatusCode, Json<crate::api::AcceptedResponse>), ApiHttpError> {
    let ApiObservedState::Live { dcs, ha, .. } = &state.observed else {
        return Err(ApiHttpError::service_unavailable(
            "state subscribers unavailable",
        ));
    };
    let response = post_switchover(
        state.cfg.scope.as_str(),
        &state.cfg.member_id,
        &state.dcs_handle,
        &dcs.latest(),
        &ha.latest(),
        request,
    )
    .await?;
    Ok((StatusCode::ACCEPTED, Json(response)))
}

async fn delete_switchover_handler(
    State(state): State<ApiRuntimeCtx<'_>>,
) -> Result<(StatusCode, Json<crate::api::AcceptedResponse>), ApiHttpError> {
    let response = delete_switchover(state.cfg.scope.as_str(), &state.dcs_handle).await?;
    Ok((StatusCode::ACCEPTED, Json(response)))
}

async fn reload_certificates(
    State(state): State<ApiRuntimeCtx<'_>>,
) -> Result<Json<ReloadCertificatesResponse>, ApiHttpError> {
    let reloaded = state
        .reload_certificates
        .reload(state.cfg)
        .await
        .map_err(|err| ApiHttpError::internal(err.to_string()))?;
    Ok(Json(reloaded))
}

async fn require_read_auth(
    State(state): State<ApiRuntimeCtx<'_>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiHttpError> {
    require_auth(state, RequiredRole::Read, request, next).await
}

async fn require_admin_auth(
    State(state): State<ApiRuntimeCtx<'_>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiHttpError> {
    require_auth(state, RequiredRole::Admin, request, next).await
}

async fn require_auth(
    state: ApiRuntimeCtx<'_>,
    required_role: RequiredRole,
    request: Request,
    next: Next,
) -> Result<Response, ApiHttpError> {
    match authorize_request(&state.cfg.api.auth, required_role, &request) {
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
    auth: &ApiAuth,
    required_role: RequiredRole,
    request: &Request,
) -> AuthDecision {
    let (read_token, admin_token) = match auth {
        ApiAuth::Disabled => return AuthDecision::Allowed,
        ApiAuth::Tokens {
            read_token,
            admin_token,
        } => (
            Some(read_token.as_str().trim()),
            Some(admin_token.as_str().trim()),
        ),
    };

    if read_token.is_none_or(str::is_empty) && admin_token.is_none_or(str::is_empty) {
        return AuthDecision::Allowed;
    }

    let Some(token) = extract_bearer_token(request) else {
        return AuthDecision::Unauthorized;
    };

    if admin_token.is_some_and(|expected| expected == token) {
        return AuthDecision::Allowed;
    }

    match required_role {
        RequiredRole::Read => {
            if read_token.is_some_and(|expected| expected == token) {
                AuthDecision::Allowed
            } else {
                AuthDecision::Unauthorized
            }
        }
        RequiredRole::Admin => {
            if read_token.is_some_and(|expected| expected == token) {
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::Path,
        process::{Child, Command},
        time::Duration,
    };

    use axum::{
        body::{to_bytes, Body},
        http::{Method, Request, StatusCode},
        Router,
    };
    use tower::util::ServiceExt;

    use crate::{
        api::{ApiCertificateReloadStep, PostgresReloadSignal, ReloadCertificatesResponse},
        config_v2::{
            runtime_test_config_with_data_dir,
            types::{ApiTransport, TlsConfig},
            RuntimeConfigV2,
        },
        dcs::DcsHandle,
        dev_support::{
            api::api_auth_from_optional_tokens, test_fs::unique_test_dir,
            tls::build_adversarial_tls_fixture,
        },
        logging::LogSender,
        process::postmaster::{lookup_managed_postmaster, ManagedPostmasterTarget},
        state::new_state_channel,
    };

    use super::{
        build_router, ApiObservedState, ApiReloadCertificatesHandle, ApiRuntimeCtx,
        ApiServerTransport,
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

    fn write_test_tls_files(
        data_dir: &Path,
        cert_pem: &str,
        key_pem: &str,
    ) -> Result<TlsConfig, String> {
        let tls_dir = data_dir.join("api-test-tls");
        fs::create_dir_all(&tls_dir)
            .map_err(|err| format!("create tls dir {} failed: {err}", tls_dir.display()))?;
        let cert = tls_dir.join("server.crt");
        let key = tls_dir.join("server.key");
        fs::write(&cert, cert_pem)
            .map_err(|err| format!("write test cert {} failed: {err}", cert.display()))?;
        fs::write(&key, key_pem)
            .map_err(|err| format!("write test key {} failed: {err}", key.display()))?;
        Ok(TlsConfig {
            cert,
            key,
            ca_cert: None,
        })
    }

    fn build_test_app(cfg: RuntimeConfigV2) -> Result<Router, String> {
        let transport = crate::tls::build_api_server_transport_v2(&cfg.api.transport)
            .map_err(|err| err.to_string())?;
        build_test_app_with_transport(cfg, transport)
    }

    fn build_test_app_with_transport(
        cfg: RuntimeConfigV2,
        transport: ApiServerTransport,
    ) -> Result<Router, String> {
        let cfg = Box::leak(Box::new(cfg));
        let (_pg_publisher, pg) = new_state_channel(crate::pginfo::state::PgInfoState::starting());
        let (_process_publisher, process) =
            new_state_channel(crate::process::state::ProcessState::starting());
        let (_dcs_publisher, dcs) = new_state_channel(crate::dcs::DcsSnapshot::starting());
        let (_ha_publisher, ha) = new_state_channel(crate::ha::state::HaState::initial(
            crate::state::WorkerStatus::Starting,
        ));
        let app = build_router(ApiRuntimeCtx {
            cfg,
            observed: ApiObservedState::Live {
                pg,
                process,
                dcs,
                ha,
            },
            dcs_handle: DcsHandle::closed(),
            transport: transport.clone(),
            reload_certificates: ApiReloadCertificatesHandle::from_transport(&transport),
            _log: LogSender::disabled(),
        })
        .map_err(|err| err.to_string())?;
        Ok(app)
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
        let ready_file = root.join("fake-postgres.ready");
        let script_contents = format!(
            "#!/bin/bash\ntrap 'printf hup >> \"{}\"' HUP\nprintf ready > \"{}\"\nwhile true; do sleep 1; done\n",
            signal_log.display(),
            ready_file.display(),
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
        wait_for_fake_postgres_ready(ready_file.as_path())?;
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

    fn wait_for_fake_postgres_ready(ready_file: &Path) -> Result<(), String> {
        let mut attempts = 0_u8;
        while attempts < 150 {
            match fs::read_to_string(ready_file) {
                Ok(contents) if !contents.is_empty() => return Ok(()),
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(format!(
                        "read fake postgres ready file {} failed: {err}",
                        ready_file.display()
                    ));
                }
            }
            std::thread::sleep(Duration::from_millis(10));
            attempts = attempts.saturating_add(1);
        }
        Err(format!(
            "fake postgres ready file {} was not written in time",
            ready_file.display()
        ))
    }

    fn wait_for_managed_postmaster_ready(data_dir: &Path) -> Result<(), String> {
        let target = ManagedPostmasterTarget::from_data_dir(data_dir.to_path_buf());
        let mut attempts = 0_u8;
        while attempts < 150 {
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
        let root = unique_test_dir("api-worker", "reload-success")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let child = spawn_fake_postgres_process(&root, &data_dir, &signal_log)?;
        let pid = child.child()?.id();
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let _child = child;
        wait_for_managed_postmaster_ready(&data_dir)?;
        let fixture = build_adversarial_tls_fixture().map_err(|err| err.to_string())?;
        let auth = api_auth_from_optional_tokens(Some("read-secret"), Some("admin-secret"))?;
        let cfg = runtime_test_config_with_data_dir(&data_dir).map_err(|err| err.to_string())?;
        let cfg = RuntimeConfigV2 {
            api: crate::config_v2::types::ApiConfig {
                auth,
                transport: ApiTransport::Https {
                    tls: write_test_tls_files(
                        data_dir.as_path(),
                        fixture.valid_server.cert_pem.as_str(),
                        fixture.valid_server.key_pem.as_str(),
                    )?,
                    client_ca: None,
                    client_cert_required: false,
                    allowed_client_common_names: Vec::new(),
                },
                ..cfg.api
            },
            ..cfg
        };
        let app = build_test_app(cfg)?;

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
        let root = unique_test_dir("api-worker", "reload-missing-postmaster")?;
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let auth = api_auth_from_optional_tokens(Some("read-secret"), Some("admin-secret"))?;
        let cfg = runtime_test_config_with_data_dir(&data_dir)
            .map(|cfg| RuntimeConfigV2 {
                api: crate::config_v2::types::ApiConfig { auth, ..cfg.api },
                ..cfg
            })
            .map_err(|err| err.to_string())?;
        let app = build_test_app(cfg)?;

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
        let root = unique_test_dir("api-worker", "reload-stale-postmaster")?;
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
        let auth = api_auth_from_optional_tokens(Some("read-secret"), Some("admin-secret"))?;
        let cfg = runtime_test_config_with_data_dir(&data_dir)
            .map(|cfg| RuntimeConfigV2 {
                api: crate::config_v2::types::ApiConfig { auth, ..cfg.api },
                ..cfg
            })
            .map_err(|err| err.to_string())?;
        let app = build_test_app(cfg)?;

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
        let root = unique_test_dir("api-worker", "reload-mismatch")?;
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
        let auth = api_auth_from_optional_tokens(Some("read-secret"), Some("admin-secret"))?;
        let cfg = runtime_test_config_with_data_dir(&target_data_dir)
            .map(|cfg| RuntimeConfigV2 {
                api: crate::config_v2::types::ApiConfig { auth, ..cfg.api },
                ..cfg
            })
            .map_err(|err| err.to_string())?;
        let app = build_test_app(cfg)?;

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
        let root = unique_test_dir("api-worker", "reload-ordering")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let child = spawn_fake_postgres_process(&root, &data_dir, &signal_log)?;
        let pid = child.child()?.id();
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let _child = child;
        wait_for_managed_postmaster_ready(&data_dir)?;
        let fixture = build_adversarial_tls_fixture().map_err(|err| err.to_string())?;
        let auth = api_auth_from_optional_tokens(Some("read-secret"), Some("admin-secret"))?;
        let good_cfg =
            runtime_test_config_with_data_dir(&data_dir).map_err(|err| err.to_string())?;
        let good_cfg = RuntimeConfigV2 {
            api: crate::config_v2::types::ApiConfig {
                auth: auth.clone(),
                transport: ApiTransport::Https {
                    tls: write_test_tls_files(
                        data_dir.as_path(),
                        fixture.valid_server.cert_pem.as_str(),
                        fixture.valid_server.key_pem.as_str(),
                    )?,
                    client_ca: None,
                    client_cert_required: false,
                    allowed_client_common_names: Vec::new(),
                },
                ..good_cfg.api
            },
            ..good_cfg
        };
        let transport = crate::tls::build_api_server_transport_v2(&good_cfg.api.transport)
            .map_err(|err| err.to_string())?;
        let cfg = runtime_test_config_with_data_dir(&data_dir).map_err(|err| err.to_string())?;
        let cfg = RuntimeConfigV2 {
            api: crate::config_v2::types::ApiConfig {
                auth,
                transport: ApiTransport::Https {
                    tls: write_test_tls_files(
                        data_dir.as_path(),
                        "not a certificate",
                        "not a key",
                    )?,
                    client_ca: None,
                    client_cert_required: false,
                    allowed_client_common_names: Vec::new(),
                },
                ..cfg.api
            },
            ..cfg
        };
        let app = build_test_app_with_transport(cfg, transport)?;

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

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn reload_certificates_returns_error_when_https_runtime_sees_http_config(
    ) -> Result<(), String> {
        let root = unique_test_dir("api-worker", "reload-transport-mismatch")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let child = spawn_fake_postgres_process(&root, &data_dir, &signal_log)?;
        let pid = child.child()?.id();
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let _child = child;
        wait_for_managed_postmaster_ready(&data_dir)?;
        let fixture = build_adversarial_tls_fixture().map_err(|err| err.to_string())?;
        let auth = api_auth_from_optional_tokens(Some("read-secret"), Some("admin-secret"))?;
        let good_cfg =
            runtime_test_config_with_data_dir(&data_dir).map_err(|err| err.to_string())?;
        let good_cfg = RuntimeConfigV2 {
            api: crate::config_v2::types::ApiConfig {
                auth: auth.clone(),
                transport: ApiTransport::Https {
                    tls: write_test_tls_files(
                        data_dir.as_path(),
                        fixture.valid_server.cert_pem.as_str(),
                        fixture.valid_server.key_pem.as_str(),
                    )?,
                    client_ca: None,
                    client_cert_required: false,
                    allowed_client_common_names: Vec::new(),
                },
                ..good_cfg.api
            },
            ..good_cfg
        };
        let transport = crate::tls::build_api_server_transport_v2(&good_cfg.api.transport)
            .map_err(|err| err.to_string())?;
        let app = build_test_app_with_transport(
            runtime_test_config_with_data_dir(&data_dir)
                .map(|cfg| RuntimeConfigV2 {
                    api: crate::config_v2::types::ApiConfig { auth, ..cfg.api },
                    ..cfg
                })
                .map_err(|err| err.to_string())?,
            transport,
        )?;

        let response = app
            .oneshot(sample_admin_request("/reload/certs")?)
            .await
            .map_err(|err| err.to_string())?;

        if response.status() != StatusCode::INTERNAL_SERVER_ERROR {
            return Err(format!("unexpected status {}", response.status()));
        }
        let body = response_body_text(response).await?;
        if !body.contains("api certificate reload requires https transport") {
            return Err(format!(
                "response body did not mention transport mismatch: {body}"
            ));
        }
        assert_no_signal_written(&signal_log).await?;
        Ok(())
    }
}
