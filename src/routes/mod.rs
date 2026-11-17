pub mod health;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::{errors::AppError, middleware::cors::cors_layer, state::AppState};

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state.config.allowed_origins);

    Router::new()
        .nest("/api/v1", api_routes())
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new().merge(health::routes())
}

async fn not_found() -> AppError {
    AppError::NotFound("Route not found".into())
}
