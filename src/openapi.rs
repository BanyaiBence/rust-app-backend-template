//! The OpenAPI document. Paths and schemas are collected from the `#[utoipa::path]`
//! annotations on the handlers as `routes::router()` registers them; this file adds only the
//! document-level metadata. See ADR 0004.

use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

/// Name of the bearer-token security scheme. `#[utoipa::path]` needs a literal, so handlers
/// spell it out: `security(("bearer" = []))`.
pub const BEARER: &str = "bearer";

#[derive(OpenApi)]
#[openapi(
    info(
        title = "backend",
        description = "JSON API. Authenticate with `POST /auth/login` and send the token as \
                       `Authorization: Bearer <token>`. Errors are `{\"code\", \"error\"}` objects.",
    ),
    tags(
        (name = "health", description = "Liveness and readiness probes"),
        (name = "auth", description = "Registration, login and the current account"),
        (name = "notes", description = "Example resource owned by the authenticated user"),
    ),
    modifiers(&BearerAuth),
)]
struct ApiDoc;

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            BEARER,
            SecurityScheme::Http(
                Http::builder()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// The document metadata, without paths; `build_app` and `openapi()` fill in the paths.
pub fn base() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// The complete OpenAPI document. Needs no database, so it can be exported at build time
/// (`make openapi`).
pub fn openapi() -> utoipa::openapi::OpenApi {
    utoipa_axum::router::OpenApiRouter::with_openapi(base())
        .merge(crate::routes::router())
        .into_openapi()
}
