use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn health_returns_ok(pool: PgPool) {
    let (status, body) = crate::common::send(
        &crate::common::app(pool),
        Method::GET,
        "/health",
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "status": "ok" }));
}

#[sqlx::test]
async fn ready_returns_ok_when_database_answers(pool: PgPool) {
    let (status, body) =
        crate::common::send(&crate::common::app(pool), Method::GET, "/ready", None, None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "status": "ok" }));
}

#[sqlx::test]
async fn ready_returns_503_when_database_is_down(pool: PgPool) {
    let app = crate::common::app(pool.clone());
    pool.close().await;

    let (status, body) = crate::common::send(&app, Method::GET, "/ready", None, None).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, json!({ "status": "unavailable" }));
}

#[sqlx::test]
async fn responses_carry_a_request_id(pool: PgPool) {
    use tower::ServiceExt;

    let app = crate::common::app(pool);
    let request = |id: Option<&str>| {
        let mut builder = axum::http::Request::builder().uri("/health");
        if let Some(id) = id {
            builder = builder.header("x-request-id", id);
        }
        builder.body(axum::body::Body::empty()).unwrap()
    };

    let generated = app.clone().oneshot(request(None)).await.unwrap();
    let id = generated.headers()["x-request-id"].to_str().unwrap();
    assert!(uuid::Uuid::parse_str(id).is_ok(), "{id}");

    let echoed = app.oneshot(request(Some("abc-123"))).await.unwrap();
    assert_eq!(echoed.headers()["x-request-id"], "abc-123");
}
