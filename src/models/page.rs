use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::error::AppError;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;

/// `?limit=&offset=` query parameters for list endpoints. See ADR 0003.
#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PageParams {
    /// Items per page, 1-100 (default 50).
    pub limit: Option<i64>,
    /// Items to skip (default 0).
    pub offset: Option<i64>,
}

impl PageParams {
    /// The effective `(limit, offset)`: defaults applied, out-of-range values rejected.
    pub fn resolve(&self) -> Result<(i64, i64), AppError> {
        let limit = self.limit.unwrap_or(DEFAULT_LIMIT);
        let offset = self.offset.unwrap_or(0);
        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(AppError::Validation(format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }
        if offset < 0 {
            return Err(AppError::Validation(
                "offset must not be negative".to_string(),
            ));
        }
        Ok((limit, offset))
    }
}

/// One page of a list. An envelope rather than a bare array, so fields such as a total can be
/// added later without breaking clients.
#[derive(Debug, Serialize, ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub limit: i64,
    pub offset: i64,
}
