use std::time::Duration;

use axum::http::{HeaderValue, Method, header};
use tower_http::cors::CorsLayer;

/// Bearer-token auth in the Authorization header: no cookies, so credentials stay disabled.
pub fn cors_layer(origins: &[HeaderValue]) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(origins.to_vec())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .max_age(Duration::from_secs(3600))
}
