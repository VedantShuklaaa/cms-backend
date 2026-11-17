pub mod assignments;
pub mod attendance;
pub mod auth;
pub mod courses;
pub mod departments;
pub mod exams;
pub mod faculty;
pub mod fees;
pub mod health;
pub mod notices;
pub mod reports;
pub mod students;
pub mod subjects;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::{errors::AppError, middleware::cors::cors_layer, state::AppState};

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state.config.allowed_origins);

    Router::new()
        .nest("/api/v1", api_routes(&state))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn api_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .merge(auth::routes(state))
        .merge(departments::routes(state))
        .merge(courses::routes(state))
        .merge(subjects::routes(state))
        .merge(faculty::routes(state))
        .merge(students::routes(state))
        .merge(attendance::routes(state))
        .merge(exams::routes(state))
        .merge(notices::routes(state))
        .merge(fees::routes(state))
        .merge(assignments::routes(state))
        .merge(reports::routes(state))
}

async fn not_found() -> AppError {
    AppError::NotFound("Route not found".into())
}
