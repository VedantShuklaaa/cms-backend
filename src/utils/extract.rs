use axum::extract::FromRequest;

use crate::errors::AppError;

/// Json extractor whose rejections use the standard error envelope.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct ApiJson<T>(pub T);
