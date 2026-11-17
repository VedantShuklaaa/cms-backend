use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;

use crate::{state::AppState, utils::response::ApiResponse};

#[derive(Serialize)]
struct HealthData {
    status: &'static str,
    database: &'static str,
    version: &'static str,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check(
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<HealthData>>) {
    let db_up = match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => true,
        Err(e) => {
            tracing::warn!(error = %e, "health check: database unreachable");
            false
        }
    };

    let (code, status, database, message) = if db_up {
        (StatusCode::OK, "ok", "up", "Service is healthy")
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "degraded",
            "down",
            "Service is degraded",
        )
    };

    let data = HealthData {
        status,
        database,
        version: env!("CARGO_PKG_VERSION"),
    };
    (code, Json(ApiResponse::ok(message, data)))
}
