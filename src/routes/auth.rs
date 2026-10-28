use axum::{Json, extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::{AuthUser, jwt, password},
    error::{AppError, ErrorBody},
    extract::AppJson,
    models::user::{Credentials, TokenResponse, UserResponse},
    repo::user_repo,
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(register))
        .routes(routes!(login))
        .routes(routes!(me))
}

/// Create an account.
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = CREATED, body = UserResponse),
        (status = CONFLICT, description = "Username already taken", body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "Invalid username or password", body = ErrorBody),
    ),
)]
pub async fn register(
    State(state): State<AppState>,
    AppJson(credentials): AppJson<Credentials>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    credentials.validate_for_registration()?;

    let password_hash = password::hash(credentials.password).await?;
    let user = user_repo::create(&state.pool, &credentials.username, &password_hash).await?;

    tracing::info!(user_id = %user.id, "user registered");
    Ok((StatusCode::CREATED, Json(user.into())))
}

/// Exchange credentials for a bearer token.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = OK, body = TokenResponse),
        (status = UNAUTHORIZED, description = "Unknown user or wrong password", body = ErrorBody),
    ),
)]
pub async fn login(
    State(state): State<AppState>,
    AppJson(credentials): AppJson<Credentials>,
) -> Result<Json<TokenResponse>, AppError> {
    let user = user_repo::find_by_username(&state.pool, &credentials.username).await?;

    // Always run a full verification so unknown usernames cannot be detected by timing.
    let (user_id, password_hash) = match user {
        Some(user) => (Some(user.id), user.password_hash),
        None => (None, password::dummy_hash()),
    };
    let password_ok = password::verify(credentials.password, password_hash).await?;

    let user_id = match (password_ok, user_id) {
        (true, Some(user_id)) => user_id,
        _ => return Err(AppError::Unauthorized),
    };

    let ttl = state.config.jwt_ttl_seconds;
    let token = jwt::issue(user_id, &state.config.jwt_secret, ttl)?;

    tracing::info!(%user_id, "user logged in");
    Ok(Json(TokenResponse {
        token,
        token_type: "Bearer",
        expires_in: ttl,
    }))
}

/// The authenticated caller's own account.
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    security(("bearer" = [])),
    responses(
        (status = OK, body = UserResponse),
        (status = UNAUTHORIZED, body = ErrorBody),
    ),
)]
pub async fn me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    // A valid token for an account that no longer exists is no longer a valid login.
    let user = user_repo::find_by_id(&state.pool, user_id)
        .await
        .map_err(|err| match err {
            AppError::NotFound => AppError::Unauthorized,
            other => other,
        })?;
    Ok(Json(user.into()))
}
