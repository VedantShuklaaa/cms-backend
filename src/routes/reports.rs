use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::get,
};

use crate::{
    handlers::reports as h,
    middleware::{
        auth::require_auth,
        rbac::{admin_only, staff_only},
    },
    state::AppState,
};

pub fn routes(state: &AppState) -> Router<AppState> {
    let admin = from_fn(admin_only);
    let staff = from_fn(staff_only);

    Router::new()
        .route(
            "/reports/overview",
            get(h::overview).route_layer(admin.clone()),
        )
        .route(
            "/reports/attendance",
            get(h::attendance).route_layer(staff.clone()),
        )
        .route("/reports/results", get(h::results).route_layer(staff))
        .route("/reports/fees", get(h::fees).route_layer(admin))
        .route_layer(from_fn_with_state(state.clone(), require_auth))
}
