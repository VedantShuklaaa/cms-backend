use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        auth::AuthUser,
        course::{Course, CourseListQuery, CreateCourseRequest, UpdateCourseRequest},
    },
    services::courses as svc,
    state::AppState,
    utils::{
        extract::{ApiJson, ApiPath, ApiQuery},
        query::Paginated,
        response::ApiResponse,
    },
};

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<CourseListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Course>>>> {
    let data = svc::list(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Courses retrieved", data)))
}

pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Course>>> {
    let data = svc::get(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Course retrieved", data)))
}

pub async fn create(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<CreateCourseRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Course>>)> {
    let data = svc::create(&state, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Course created", data)),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateCourseRequest>,
) -> AppResult<Json<ApiResponse<Course>>> {
    let data = svc::update(&state, id, req).await?;
    Ok(Json(ApiResponse::ok("Course updated", data)))
}

pub async fn remove(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate(&state, id).await?;
    Ok(Json(ApiResponse::ok("Course deactivated", ())))
}
