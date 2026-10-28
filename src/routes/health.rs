use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(health))
        .routes(routes!(ready))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// `ok`, or `unavailable` when a dependency is down.
    #[schema(example = "ok")]
    pub status: &'static str,
}

/// Liveness: the process is up and serving HTTP. Never touches the database, so a database
/// outage does not get the container restarted.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = OK, body = HealthResponse)),
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// Readiness: the database answers, so requests can be served. For load balancers and
/// orchestrators deciding whether to route traffic here.
#[utoipa::path(
    get,
    path = "/ready",
    tag = "health",
    responses(
        (status = OK, body = HealthResponse),
        (status = SERVICE_UNAVAILABLE, body = HealthResponse),
    ),
)]
pub async fn ready(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (StatusCode::OK, Json(HealthResponse { status: "ok" })),
        Err(err) => {
            tracing::warn!(error = %err, "readiness check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unavailable",
                }),
            )
        }
    }
}
