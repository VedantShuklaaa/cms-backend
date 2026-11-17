use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post, put},
};

use crate::{
    handlers::exams as h,
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
            "/exams",
            get(h::list_exams).merge(post(h::create_exam).route_layer(admin.clone())),
        )
        .route(
            "/exams/{id}",
            get(h::get_exam).merge(
                put(h::update_exam)
                    .delete(h::remove_exam)
                    .route_layer(admin.clone()),
            ),
        )
        .route(
            "/results",
            get(h::list_results)
                .route_layer(staff.clone())
                .merge(post(h::create_result).route_layer(staff.clone())),
        )
        .route("/results/me", get(h::my_results))
        .route(
            "/results/{id}",
            get(h::get_result).merge(
                put(h::update_result)
                    .delete(h::remove_result)
                    .route_layer(staff),
            ),
        )
        .route_layer(from_fn_with_state(state.clone(), require_auth))
}
