use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, post, put},
};

use crate::{
    handlers::subjects as h,
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
            "/subjects",
            get(h::list).merge(post(h::create).route_layer(admin.clone())),
        )
        .route("/subjects/mine", get(h::mine))
        .route(
            "/subjects/{id}",
            get(h::get_one).merge(put(h::update).delete(h::remove).route_layer(admin.clone())),
        )
        .route(
            "/subjects/{id}/faculty",
            get(h::list_faculty)
                .route_layer(staff)
                .merge(post(h::assign_faculty).route_layer(admin.clone())),
        )
        .route(
            "/subjects/{id}/faculty/{faculty_id}",
            delete(h::unassign_faculty).route_layer(admin),
        )
        .route_layer(from_fn_with_state(state.clone(), require_auth))
}
