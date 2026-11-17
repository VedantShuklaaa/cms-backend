use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::{handlers::auth as h, middleware::auth::require_auth, state::AppState};

pub fn routes(state: &AppState) -> Router<AppState> {
    let authenticated = Router::new()
        .route("/auth/logout", post(h::logout))
        .route("/auth/me", get(h::me))
        .route("/auth/change-password", post(h::change_password))
        .route_layer(from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .route("/auth/login", post(h::login))
        .merge(authenticated)
}
