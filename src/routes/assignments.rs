use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post, put},
};

use crate::{
    handlers::assignments as h,
    middleware::{auth::require_auth, rbac::staff_only},
    state::AppState,
};

pub fn routes(state: &AppState) -> Router<AppState> {
    let staff = from_fn(staff_only);

    Router::new()
        .route(
            "/assignments",
            get(h::list).merge(post(h::create).route_layer(staff.clone())),
        )
        .route(
            "/assignments/{id}",
            get(h::get_one).merge(put(h::update).delete(h::remove).route_layer(staff.clone())),
        )
        .route(
            "/assignments/{id}/submissions",
            get(h::list_submissions)
                .route_layer(staff.clone())
                .merge(post(h::submit)),
        )
        .route(
            "/assignments/{id}/submissions/{submission_id}",
            put(h::grade_submission)
                .delete(h::remove_submission)
                .route_layer(staff),
        )
        .route("/submissions/me", get(h::my_submissions))
        .route_layer(from_fn_with_state(state.clone(), require_auth))
}
