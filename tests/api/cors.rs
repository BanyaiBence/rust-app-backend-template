use axum::{
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header},
};
use sqlx::PgPool;
use tower::ServiceExt;

use crate::common::ALLOWED_ORIGIN;

/// Sends a CORS preflight for `POST /auth/login` from `origin` and returns the response headers.
async fn preflight(pool: PgPool, origin: &str) -> (StatusCode, HeaderMap) {
    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/auth/login")
        .header(header::ORIGIN, origin)
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "content-type")
        .body(Body::empty())
        .unwrap();
    let response = crate::common::app(pool).oneshot(request).await.unwrap();
    (response.status(), response.headers().clone())
}

#[sqlx::test]
async fn preflight_from_allowed_origin_is_permitted(pool: PgPool) {
    let (status, headers) = preflight(pool, ALLOWED_ORIGIN).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], ALLOWED_ORIGIN);
    let methods = headers[header::ACCESS_CONTROL_ALLOW_METHODS]
        .to_str()
        .unwrap();
    assert!(methods.contains("POST"), "allowed methods: {methods}");
}

#[sqlx::test]
async fn preflight_from_other_origin_gets_no_allow_header(pool: PgPool) {
    let (_, headers) = preflight(pool, "https://evil.example.com").await;

    assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
}
