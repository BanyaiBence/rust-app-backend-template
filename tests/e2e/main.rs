//! End-to-end smoke tests against a running deployment (by default the docker compose stack).
//!
//! Ignored by default because they need the stack to be up:
//!
//! ```sh
//! make up
//! make test-e2e
//! ```
//!
//! Set `E2E_BASE_URL` to target another deployment.

use reqwest::{Client, StatusCode};
use serde_json::{Value, json};

fn base_url() -> String {
    std::env::var("E2E_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

fn url(path: &str) -> String {
    format!("{}{path}", base_url())
}

#[tokio::test]
#[ignore = "requires the docker compose stack"]
async fn health_endpoint_is_up() {
    let response = reqwest::get(url("/health")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.json::<Value>().await.unwrap(),
        json!({ "status": "ok" })
    );
}

#[tokio::test]
#[ignore = "requires the docker compose stack"]
async fn register_login_and_manage_notes() {
    let client = Client::new();
    // Unique per run so the test can be repeated against a persistent database.
    let username = format!("e2e_{}", &uuid::Uuid::new_v4().simple().to_string()[..12]);
    let credentials = json!({ "username": username, "password": "password123" });

    let response = client
        .post(url("/auth/register"))
        .json(&credentials)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let login: Value = client
        .post(url("/auth/login"))
        .json(&credentials)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = login["token"].as_str().expect("login returns a token");

    let response = client
        .post(url("/notes"))
        .bearer_auth(token)
        .json(&json!({ "title": "ship it", "body": "via docker" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let note: Value = response.json().await.unwrap();
    let note_url = url(&format!("/notes/{}", note["id"].as_str().unwrap()));

    let response = client
        .patch(&note_url)
        .bearer_auth(token)
        .json(&json!({ "title": "shipped" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.json::<Value>().await.unwrap()["title"], "shipped");

    let page: Value = client
        .get(url("/notes"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 1);

    let response = client
        .delete(&note_url)
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = client
        .get(&note_url)
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
