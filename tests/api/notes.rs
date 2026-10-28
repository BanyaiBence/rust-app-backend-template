use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{register_and_login, send};

async fn create_note(app: &Router, token: &str, title: &str) -> Value {
    let (status, body) = send(
        app,
        Method::POST,
        "/notes",
        Some(json!({ "title": title, "body": "text" })),
        Some(token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body
}

#[sqlx::test]
async fn notes_require_authentication(pool: PgPool) {
    let app = crate::common::app(pool);

    let (status, body) = send(&app, Method::GET, "/notes", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthorized");
}

#[sqlx::test]
async fn create_get_update_delete_round_trip(pool: PgPool) {
    let app = crate::common::app(pool);
    let token = register_and_login(&app, "alice").await;

    let note = create_note(&app, &token, "first").await;
    let uri = format!("/notes/{}", note["id"].as_str().unwrap());
    assert_eq!(note["title"], "first");
    assert_eq!(note["body"], "text");

    let (status, fetched) = send(&app, Method::GET, &uri, None, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched, note);

    let (status, updated) = send(
        &app,
        Method::PATCH,
        &uri,
        Some(json!({ "title": "renamed" })),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["title"], "renamed");
    assert_eq!(updated["body"], "text");

    let (status, body) = send(&app, Method::DELETE, &uri, None, Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(body, Value::Null);

    let (status, _) = send(&app, Method::GET, &uri, None, Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn list_returns_a_page_envelope(pool: PgPool) {
    let app = crate::common::app(pool);
    let token = register_and_login(&app, "alice").await;
    for title in ["one", "two", "three"] {
        create_note(&app, &token, title).await;
    }

    let (status, page) = send(&app, Method::GET, "/notes?limit=2", None, Some(&token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(page["limit"], 2);
    assert_eq!(page["offset"], 0);
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert_eq!(page["items"][0]["title"], "three");
}

#[sqlx::test]
async fn invalid_paging_is_unprocessable(pool: PgPool) {
    let app = crate::common::app(pool);
    let token = register_and_login(&app, "alice").await;

    for uri in ["/notes?limit=0", "/notes?limit=abc", "/notes?offset=-1"] {
        let (status, body) = send(&app, Method::GET, uri, None, Some(&token)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{uri}");
        assert_eq!(body["code"], "validation_failed", "{uri}");
    }
}

#[sqlx::test]
async fn blank_title_is_unprocessable(pool: PgPool) {
    let app = crate::common::app(pool);
    let token = register_and_login(&app, "alice").await;

    let (status, body) = send(
        &app,
        Method::POST,
        "/notes",
        Some(json!({ "title": " " })),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body,
        json!({ "code": "validation_failed", "error": "title must not be empty" })
    );
}

#[sqlx::test]
async fn users_cannot_see_each_others_notes(pool: PgPool) {
    let app = crate::common::app(pool);
    let alice = register_and_login(&app, "alice").await;
    let bob = register_and_login(&app, "bob").await;
    let note = create_note(&app, &alice, "private").await;
    let uri = format!("/notes/{}", note["id"].as_str().unwrap());

    let (status, _) = send(&app, Method::GET, &uri, None, Some(&bob)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = send(&app, Method::DELETE, &uri, None, Some(&bob)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, page) = send(&app, Method::GET, "/notes", None, Some(&bob)).await;
    assert_eq!(page["items"], json!([]));
}

#[sqlx::test]
async fn malformed_id_is_not_found(pool: PgPool) {
    let app = crate::common::app(pool);
    let token = register_and_login(&app, "alice").await;

    let (status, _) = send(&app, Method::GET, "/notes/not-a-uuid", None, Some(&token)).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
