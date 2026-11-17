use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, post, put},
};

use crate::{
    handlers::fees as h,
    middleware::{auth::require_auth, rbac::admin_only},
    state::AppState,
};

pub fn routes(state: &AppState) -> Router<AppState> {
    let admin = from_fn(admin_only);

    Router::new()
        .route(
            "/fees",
            get(h::list).merge(post(h::create).route_layer(admin.clone())),
        )
        .route("/fees/me", get(h::my_statements))
        .route("/fees/me/payments", get(h::my_payments))
        .route(
            "/fees/{id}",
            get(h::get_one).merge(put(h::update).delete(h::remove).route_layer(admin.clone())),
        )
        .route(
            "/fees/{id}/payments",
            get(h::list_payments)
                .route_layer(admin.clone())
                .merge(post(h::create_payment).route_layer(admin.clone())),
        )
        .route(
            "/fees/{id}/payments/{payment_id}",
            delete(h::remove_payment).route_layer(admin),
        )
        .route_layer(from_fn_with_state(state.clone(), require_auth))
}
