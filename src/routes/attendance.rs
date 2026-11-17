use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post, put},
};

use crate::{
    handlers::attendance as h,
    middleware::{auth::require_auth, rbac::staff_only},
    state::AppState,
};

pub fn routes(state: &AppState) -> Router<AppState> {
    let staff = from_fn(staff_only);

    Router::new()
        .route(
            "/attendance",
            get(h::list)
                .route_layer(staff.clone())
                .merge(post(h::create).route_layer(staff.clone())),
        )
        .route("/attendance/me", get(h::mine))
        .route(
            "/attendance/{id}",
            get(h::get_one).merge(put(h::update).delete(h::remove).route_layer(staff)),
        )
        .route_layer(from_fn_with_state(state.clone(), require_auth))
}
