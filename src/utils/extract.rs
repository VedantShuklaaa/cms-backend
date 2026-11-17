use axum::extract::{FromRequest, FromRequestParts};

use crate::errors::AppError;

/// Json extractor whose rejections use the standard error envelope.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct ApiJson<T>(pub T);

/// Query-string extractor with the standard error envelope.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(AppError))]
pub struct ApiQuery<T>(pub T);

/// Path extractor with the standard error envelope.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(AppError))]
pub struct ApiPath<T>(pub T);
