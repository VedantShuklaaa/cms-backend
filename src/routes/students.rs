use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post, put},
};

use crate::{
    handlers::students as h,
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
            "/students",
            get(h::list)
                .route_layer(staff)
                .merge(post(h::create).route_layer(admin.clone())),
        )
        .route("/students/me", get(h::me))
        .route(
            "/students/{id}",
            get(h::get_one).merge(put(h::update).delete(h::remove).route_layer(admin)),
        )
        .route_layer(from_fn_with_state(state.clone(), require_auth))
}
