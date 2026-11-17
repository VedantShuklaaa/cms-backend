use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
};

use crate::{
    handlers::auth as h,
    middleware::{auth::require_auth, rbac::admin_only},
    state::AppState,
};

pub fn routes(state: &AppState) -> Router<AppState> {
    // Later route_layer calls wrap earlier ones, so require_auth runs first.
    let authenticated = Router::new()
        .route("/auth/logout", post(h::logout))
        .route("/auth/me", get(h::me))
        .route("/auth/change-password", post(h::change_password))
        .route_layer(from_fn_with_state(state.clone(), require_auth));

    let admin = Router::new()
        .route("/auth/admin-ping", get(h::admin_ping)) // temporary, remove in Phase 2
        .route_layer(from_fn(admin_only))
        .route_layer(from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .route("/auth/login", post(h::login))
        .merge(authenticated)
        .merge(admin)
}
