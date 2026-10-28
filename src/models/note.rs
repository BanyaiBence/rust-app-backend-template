use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::AppError;

const TITLE_MAX_CHARS: usize = 200;
const BODY_MAX_CHARS: usize = 10_000;

/// A note as stored and as returned by the API.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNote {
    /// 1-200 characters, not only whitespace.
    #[schema(example = "Groceries")]
    pub title: String,
    /// Up to 10 000 characters; empty when omitted.
    #[serde(default)]
    #[schema(example = "milk, eggs")]
    pub body: String,
}

impl CreateNote {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_title(&self.title)?;
        validate_body(&self.body)
    }
}

/// A partial update: omitted fields keep their current value.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub body: Option<String>,
}

impl UpdateNote {
    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(title) = &self.title {
            validate_title(title)?;
        }
        if let Some(body) = &self.body {
            validate_body(body)?;
        }
        Ok(())
    }
}

fn validate_title(title: &str) -> Result<(), AppError> {
    if title.trim().is_empty() {
        return Err(AppError::Validation("title must not be empty".to_string()));
    }
    if title.chars().count() > TITLE_MAX_CHARS {
        return Err(AppError::Validation(format!(
            "title must be at most {TITLE_MAX_CHARS} characters"
        )));
    }
    Ok(())
}

fn validate_body(body: &str) -> Result<(), AppError> {
    if body.chars().count() > BODY_MAX_CHARS {
        return Err(AppError::Validation(format!(
            "body must be at most {BODY_MAX_CHARS} characters"
        )));
    }
    Ok(())
}
