use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

/// Every error response body: a stable machine-readable `code` for clients to branch on,
/// and a human-readable `error` message that may change wording. See ADR 0002.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    /// One of `not_found`, `unauthorized`, `conflict`, `validation_failed`,
    /// `payload_too_large`, `internal`.
    #[schema(example = "not_found")]
    pub code: &'static str,
    /// Human-readable description; wording may change.
    #[schema(example = "not found")]
    pub error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Validation(String),
    #[error("request body too large")]
    PayloadTooLarge,
    #[error(transparent)]
    Database(sqlx::Error),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::Database(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// The `code` field of the response. Part of the API contract: never change an existing
    /// code, only add new ones.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NotFound => "not_found",
            AppError::Unauthorized => "unauthorized",
            AppError::Conflict(_) => "conflict",
            AppError::Validation(_) => "validation_failed",
            AppError::PayloadTooLarge => "payload_too_large",
            AppError::Database(_) | AppError::Internal(_) => "internal",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();

        // Never expose server-side failure details to clients; log them instead.
        let message = if status.is_server_error() {
            tracing::error!(error = %self, "request failed");
            "internal server error".to_string()
        } else {
            self.to_string()
        };

        let body = ErrorBody {
            code: self.code(),
            error: message,
        };
        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound,
            err if err
                .as_database_error()
                .is_some_and(|db| db.is_unique_violation()) =>
            {
                AppError::Conflict("resource already exists".to_string())
            }
            err => AppError::Database(err),
        }
    }
}
