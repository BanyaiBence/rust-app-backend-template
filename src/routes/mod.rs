//! HTTP handlers, one module per resource. Each module's `router()` registers its handlers with
//! `routes!`, which also adds them to the OpenAPI document; `router()` here combines them.

use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

pub mod auth;
pub mod health;
pub mod notes;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .merge(health::router())
        .merge(auth::router())
        .merge(notes::router())
}
