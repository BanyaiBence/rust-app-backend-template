pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod extract;
pub mod models;
pub mod openapi;
pub mod repo;
pub mod routes;
pub mod state;
pub mod telemetry;

use std::time::Duration;

use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::{HeaderName, HeaderValue, Method, Request, header},
    routing::get,
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{config::Config, state::AppState};

/// Largest accepted request body. JSON payloads here are small; this leaves ample headroom.
pub const MAX_BODY_BYTES: usize = 64 * 1024;

/// Correlates the log lines of one request; a client-supplied value is kept, so a request can
/// be traced across services.
pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

pub fn build_app(state: AppState) -> Router {
    let cors = cors_layer(&state.config);

    let (api, spec) = OpenApiRouter::with_openapi(openapi::base())
        .merge(routes::router())
        .split_for_parts();

    api.route("/openapi.json", get(Json(spec.clone())))
        .merge(Scalar::with_url("/docs", spec))
        // Layers wrap everything added before them, so a request passes through them bottom
        // to top: request id first, then tracing (which logs the id), then CORS.
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(cors)
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(request_span)
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
        .with_state(state)
}

/// The span every log line of a request is recorded in.
fn request_span<B>(request: &Request<B>) -> tracing::Span {
    let request_id = request
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    tracing::info_span!(
        "request",
        method = %request.method(),
        uri = %request.uri(),
        request_id,
    )
}

/// Allows browser clients on the configured origins. With no origins configured the layer
/// matches nothing, so cross-origin browser requests are refused.
fn cors_layer(config: &Config) -> CorsLayer {
    let origins = if config.cors_allowed_origins == ["*"] {
        AllowOrigin::any()
    } else {
        AllowOrigin::list(
            config
                .cors_allowed_origins
                .iter()
                .filter_map(|origin| HeaderValue::from_str(origin).ok()),
        )
    };

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            REQUEST_ID_HEADER,
        ])
        .expose_headers([REQUEST_ID_HEADER])
        .max_age(Duration::from_secs(3600))
}
