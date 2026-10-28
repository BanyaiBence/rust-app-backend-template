use axum::{Json, extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{AppError, ErrorBody},
    extract::{AppJson, AppPath, AppQuery},
    models::{
        note::{CreateNote, Note, UpdateNote},
        page::{Page, PageParams},
    },
    repo::note_repo,
    state::AppState,
};

/// Handlers sharing a path go in one `routes!` call; a path may only be registered once.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(get, update, delete))
}

/// List the caller's notes, newest first.
#[utoipa::path(
    get,
    path = "/notes",
    tag = "notes",
    security(("bearer" = [])),
    params(PageParams),
    responses(
        (status = OK, body = Page<Note>),
        (status = UNAUTHORIZED, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, description = "Invalid paging parameters", body = ErrorBody),
    ),
)]
pub async fn list(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    AppQuery(params): AppQuery<PageParams>,
) -> Result<Json<Page<Note>>, AppError> {
    let (limit, offset) = params.resolve()?;
    let items = note_repo::list(&state.pool, user_id, limit, offset).await?;
    Ok(Json(Page {
        items,
        limit,
        offset,
    }))
}

/// Create a note.
#[utoipa::path(
    post,
    path = "/notes",
    tag = "notes",
    security(("bearer" = [])),
    request_body = CreateNote,
    responses(
        (status = CREATED, body = Note),
        (status = UNAUTHORIZED, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    AppJson(new): AppJson<CreateNote>,
) -> Result<(StatusCode, Json<Note>), AppError> {
    new.validate()?;
    let note = note_repo::create(&state.pool, user_id, &new).await?;
    Ok((StatusCode::CREATED, Json(note)))
}

/// Fetch one of the caller's notes.
#[utoipa::path(
    get,
    path = "/notes/{id}",
    tag = "notes",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Note id")),
    responses(
        (status = OK, body = Note),
        (status = UNAUTHORIZED, body = ErrorBody),
        (status = NOT_FOUND, description = "No such note, or it belongs to another user", body = ErrorBody),
    ),
)]
pub async fn get(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    AppPath(id): AppPath<Uuid>,
) -> Result<Json<Note>, AppError> {
    Ok(Json(note_repo::find(&state.pool, user_id, id).await?))
}

/// Change some fields of a note; omitted fields are left as they are.
#[utoipa::path(
    patch,
    path = "/notes/{id}",
    tag = "notes",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Note id")),
    request_body = UpdateNote,
    responses(
        (status = OK, body = Note),
        (status = UNAUTHORIZED, body = ErrorBody),
        (status = NOT_FOUND, body = ErrorBody),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody),
    ),
)]
pub async fn update(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    AppPath(id): AppPath<Uuid>,
    AppJson(changes): AppJson<UpdateNote>,
) -> Result<Json<Note>, AppError> {
    changes.validate()?;
    Ok(Json(
        note_repo::update(&state.pool, user_id, id, &changes).await?,
    ))
}

/// Delete a note.
#[utoipa::path(
    delete,
    path = "/notes/{id}",
    tag = "notes",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "Note id")),
    responses(
        (status = NO_CONTENT),
        (status = UNAUTHORIZED, body = ErrorBody),
        (status = NOT_FOUND, body = ErrorBody),
    ),
)]
pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    AppPath(id): AppPath<Uuid>,
) -> Result<StatusCode, AppError> {
    note_repo::delete(&state.pool, user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
