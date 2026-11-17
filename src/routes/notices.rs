use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post, put},
};

use crate::{
    handlers::notices as h,
    middleware::{auth::require_auth, rbac::admin_only},
    state::AppState,
};

pub fn routes(state: &AppState) -> Router<AppState> {
    let admin = from_fn(admin_only);

    Router::new()
        .route(
            "/notices",
            get(h::list).merge(post(h::create).route_layer(admin.clone())),
        )
        .route(
            "/notices/{id}",
            get(h::get_one).merge(put(h::update).delete(h::remove).route_layer(admin)),
        )
        .route_layer(from_fn_with_state(state.clone(), require_auth))
}
