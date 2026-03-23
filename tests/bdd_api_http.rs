use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use pgtuskmaster_test_support::api::{build_test_router, build_test_router_with_live_state};
use tower::util::ServiceExt;

fn request(
    method: Method,
    uri: &str,
    bearer_token: Option<&str>,
) -> Result<Request<Body>, axum::http::Error> {
    let builder = Request::builder().method(method).uri(uri);
    let builder = match bearer_token {
        Some(token) => builder.header("authorization", format!("Bearer {token}")),
        None => builder,
    };
    builder.body(Body::empty())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_state_requires_live_state_subscribers() -> Result<(), Box<dyn std::error::Error>> {
    let app = build_test_router(None, None)?;

    let response = app.oneshot(request(Method::GET, "/state", None)?).await?;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_old_debug_and_fallback_routes_are_gone() -> Result<(), Box<dyn std::error::Error>>
{
    let app = build_test_router(None, None)?;

    let debug_response = app
        .clone()
        .oneshot(request(Method::GET, "/debug/verbose", None)?)
        .await?;
    assert_eq!(debug_response.status(), StatusCode::NOT_FOUND);

    let fallback_response = app
        .oneshot(request(Method::GET, "/fallback/cluster", None)?)
        .await?;
    assert_eq!(fallback_response.status(), StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_auth_token_denies_missing_header() -> Result<(), Box<dyn std::error::Error>> {
    let app = build_test_router(Some("reader"), Some("admin"))?;

    let response = app.oneshot(request(Method::GET, "/state", None)?).await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_auth_token_denies_invalid_header() -> Result<(), Box<dyn std::error::Error>> {
    let app = build_test_router(Some("reader"), Some("admin"))?;

    let response = app
        .oneshot(request(Method::GET, "/state", Some("wrong-token"))?)
        .await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_state_succeeds_with_live_subscribers() -> Result<(), Box<dyn std::error::Error>> {
    let app = build_test_router_with_live_state(None, None)?;

    let response = app.oneshot(request(Method::GET, "/state", None)?).await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_read_token_can_read_but_not_call_admin_routes(
) -> Result<(), Box<dyn std::error::Error>> {
    let app = build_test_router_with_live_state(Some("reader"), Some("admin"))?;

    let read_response = app
        .clone()
        .oneshot(request(Method::GET, "/state", Some("reader"))?)
        .await?;
    assert_eq!(read_response.status(), StatusCode::OK);

    let admin_response = app
        .oneshot(request(Method::POST, "/reload/certs", Some("reader"))?)
        .await?;
    assert_eq!(admin_response.status(), StatusCode::FORBIDDEN);
    Ok(())
}
