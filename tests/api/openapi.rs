use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test]
async fn spec_documents_every_route(pool: PgPool) {
    let (status, spec) = crate::common::send(
        &crate::common::app(pool),
        Method::GET,
        "/openapi.json",
        None,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let paths = spec["paths"].as_object().unwrap();
    let mut documented: Vec<_> = paths.keys().map(String::as_str).collect();
    documented.sort_unstable();
    assert_eq!(
        documented,
        [
            "/auth/login",
            "/auth/me",
            "/auth/register",
            "/health",
            "/notes",
            "/notes/{id}",
            "/ready"
        ]
    );
    assert_eq!(
        spec["components"]["securitySchemes"]["bearer"]["scheme"],
        "bearer"
    );
    assert_eq!(
        spec["paths"]["/notes"]["get"]["security"][0]["bearer"],
        serde_json::json!([])
    );
}

#[sqlx::test]
async fn served_spec_matches_exported_spec(pool: PgPool) {
    let (_, served) = crate::common::send(
        &crate::common::app(pool),
        Method::GET,
        "/openapi.json",
        None,
        None,
    )
    .await;

    assert_eq!(
        served,
        serde_json::to_value(backend::openapi::openapi()).unwrap()
    );
}

#[sqlx::test]
async fn docs_page_is_html(pool: PgPool) {
    let request = Request::builder().uri("/docs").body(Body::empty()).unwrap();
    let response = crate::common::app(pool).oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response.headers()[header::CONTENT_TYPE].to_str().unwrap();
    assert!(content_type.starts_with("text/html"), "{content_type}");
}
