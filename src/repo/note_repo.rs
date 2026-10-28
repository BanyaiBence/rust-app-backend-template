//! Every query takes the owner's id and filters on it, so another user's note is
//! indistinguishable from a missing one (`NotFound`).

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::note::{CreateNote, Note, UpdateNote},
};

#[tracing::instrument(skip(pool, new))]
pub async fn create(pool: &PgPool, user_id: Uuid, new: &CreateNote) -> Result<Note, AppError> {
    let note = sqlx::query_as!(
        Note,
        r#"
        INSERT INTO notes (id, user_id, title, body)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, title, body, created_at, updated_at
        "#,
        Uuid::new_v4(),
        user_id,
        new.title,
        new.body,
    )
    .fetch_one(pool)
    .await?;
    Ok(note)
}

/// The owner's notes, newest first.
#[tracing::instrument(skip(pool))]
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Note>, AppError> {
    let notes = sqlx::query_as!(
        Note,
        r#"
        SELECT id, user_id, title, body, created_at, updated_at
        FROM notes
        WHERE user_id = $1
        ORDER BY created_at DESC, id
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;
    Ok(notes)
}

#[tracing::instrument(skip(pool))]
pub async fn find(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<Note, AppError> {
    let note = sqlx::query_as!(
        Note,
        r#"
        SELECT id, user_id, title, body, created_at, updated_at
        FROM notes
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(note)
}

/// Applies the fields present in `changes`; absent fields keep their stored value.
#[tracing::instrument(skip(pool, changes))]
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    changes: &UpdateNote,
) -> Result<Note, AppError> {
    let note = sqlx::query_as!(
        Note,
        r#"
        UPDATE notes
        SET title = COALESCE($3, title),
            body = COALESCE($4, body),
            updated_at = now()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, title, body, created_at, updated_at
        "#,
        id,
        user_id,
        changes.title.as_deref(),
        changes.body.as_deref(),
    )
    .fetch_one(pool)
    .await?;
    Ok(note)
}

#[tracing::instrument(skip(pool))]
pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!(
        "DELETE FROM notes WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
