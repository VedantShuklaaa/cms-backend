use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Validation(String),
    #[error("database error: {0}")]
    Database(sqlx::Error),
    #[error("{0}")]
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".into()),
            sqlx::Error::Database(ref db) if db.is_unique_violation() => {
                AppError::Conflict("Resource already exists".into())
            }
            sqlx::Error::Database(ref db) if db.is_foreign_key_violation() => {
                AppError::Conflict("Operation conflicts with related records".into())
            }
            sqlx::Error::Database(ref db) if db.is_check_violation() => {
                AppError::BadRequest("Value violates a data constraint".into())
            }
            other => AppError::Database(other),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    success: bool,
    message: String,
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, error) = match &self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "Invalid request", m.clone()),
            AppError::Unauthorized(m) => (
                StatusCode::UNAUTHORIZED,
                "Authentication required",
                m.clone(),
            ),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, "Access denied", m.clone()),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "Resource not found", m.clone()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, "Conflict", m.clone()),
            AppError::Validation(m) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation failed",
                m.clone(),
            ),
            AppError::Database(e) => {
                tracing::error!(error = %e, "database error");
                internal()
            }
            AppError::Internal(m) => {
                tracing::error!(error = %m, "internal error");
                internal()
            }
        };

        let body = ErrorBody {
            success: false,
            message: message.to_string(),
            error,
        };
        (status, Json(body)).into_response()
    }
}

// Internal details are logged, never sent to the client.
fn internal() -> (StatusCode, &'static str, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error",
        "An unexpected error occurred".to_string(),
    )
}

impl From<axum::extract::rejection::JsonRejection> for AppError {
    fn from(r: axum::extract::rejection::JsonRejection) -> Self {
        AppError::BadRequest(r.body_text())
    }
}

impl From<axum::extract::rejection::QueryRejection> for AppError {
    fn from(r: axum::extract::rejection::QueryRejection) -> Self {
        AppError::BadRequest(r.body_text())
    }
}

impl From<axum::extract::rejection::PathRejection> for AppError {
    fn from(r: axum::extract::rejection::PathRejection) -> Self {
        AppError::BadRequest(r.body_text())
    }
}

/// Maps a unique-constraint violation to a specific 409 message; everything else uses the default mapping.
pub fn conflict_on_unique(e: sqlx::Error, msg: &str) -> AppError {
    match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => AppError::Conflict(msg.into()),
        _ => e.into(),
    }
}
