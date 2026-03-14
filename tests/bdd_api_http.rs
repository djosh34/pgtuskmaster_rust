use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use pgtuskmaster_rust::{
    config::{ApiAuthConfig, ApiRoleTokensConfig, RuntimeConfig, SecretSource},
    dcs::DcsHandle,
};
use pgtuskmaster_test_support::api::{build_test_router, build_test_router_with_live_state};
use tower::util::ServiceExt;

fn sample_runtime_config(read_token: Option<&str>, admin_token: Option<&str>) -> RuntimeConfig {
    let auth = match (read_token, admin_token) {
        (None, None) => ApiAuthConfig::Disabled,
        (read_token, admin_token) => ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: read_token.map(|token| SecretSource::Inline {
                content: token.to_string(),
            }),
            admin_token: admin_token.map(|token| SecretSource::Inline {
                content: token.to_string(),
            }),
        }),
    };

    pgtuskmaster_test_support::runtime_config::RuntimeConfigBuilder::new()
        .with_api_auth(auth)
        .build()
}

fn request(
    method: Method,
    uri: &str,
    bearer_token: Option<&str>,
) -> Result<Request<Body>, String> {
    let builder = Request::builder().method(method).uri(uri);
    let builder = match bearer_token {
        Some(token) => builder.header("authorization", format!("Bearer {token}")),
        None => builder,
    };
    builder.body(Body::empty()).map_err(|err| err.to_string())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_state_requires_live_state_subscribers() -> Result<(), String> {
    let app = build_test_router(sample_runtime_config(None, None), DcsHandle::closed())
        .map_err(|err| err.to_string())?;

    let response = app
        .oneshot(request(Method::GET, "/state", None)?)
        .await
        .map_err(|err| err.to_string())?;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_old_debug_and_fallback_routes_are_gone() -> Result<(), String> {
    let app = build_test_router(sample_runtime_config(None, None), DcsHandle::closed())
        .map_err(|err| err.to_string())?;

    let debug_response = app
        .clone()
        .oneshot(request(Method::GET, "/debug/verbose", None)?)
        .await
        .map_err(|err| err.to_string())?;
    assert_eq!(debug_response.status(), StatusCode::NOT_FOUND);

    let fallback_response = app
        .oneshot(request(Method::GET, "/fallback/cluster", None)?)
        .await
        .map_err(|err| err.to_string())?;
    assert_eq!(fallback_response.status(), StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_auth_token_denies_missing_header() -> Result<(), String> {
    let app = build_test_router(
        sample_runtime_config(Some("reader"), Some("admin")),
        DcsHandle::closed(),
    )
        .map_err(|err| err.to_string())?;

    let response = app
        .oneshot(request(Method::GET, "/state", None)?)
        .await
        .map_err(|err| err.to_string())?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_auth_token_denies_invalid_header() -> Result<(), String> {
    let app = build_test_router(
        sample_runtime_config(Some("reader"), Some("admin")),
        DcsHandle::closed(),
    )
    .map_err(|err| err.to_string())?;

    let response = app
        .oneshot(request(Method::GET, "/state", Some("wrong-token"))?)
        .await
        .map_err(|err| err.to_string())?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_state_succeeds_with_live_subscribers() -> Result<(), String> {
    let app = build_test_router_with_live_state(
        sample_runtime_config(None, None),
        DcsHandle::closed(),
    )
    .map_err(|err| err.to_string())?;

    let response = app
        .oneshot(request(Method::GET, "/state", None)?)
        .await
        .map_err(|err| err.to_string())?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_read_token_can_read_but_not_call_admin_routes() -> Result<(), String> {
    let app = build_test_router_with_live_state(
        sample_runtime_config(Some("reader"), Some("admin")),
        DcsHandle::closed(),
    )
    .map_err(|err| err.to_string())?;

    let read_response = app
        .clone()
        .oneshot(request(Method::GET, "/state", Some("reader"))?)
        .await
        .map_err(|err| err.to_string())?;
    assert_eq!(read_response.status(), StatusCode::OK);

    let admin_response = app
        .oneshot(request(Method::POST, "/reload/certs", Some("reader"))?)
        .await
        .map_err(|err| err.to_string())?;
    assert_eq!(admin_response.status(), StatusCode::FORBIDDEN);
    Ok(())
}
