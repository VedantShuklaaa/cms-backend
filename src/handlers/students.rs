use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        auth::AuthUser,
        student::{CreateStudentRequest, Student, StudentListQuery, UpdateStudentRequest},
    },
    services::students as svc,
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
    ApiQuery(q): ApiQuery<StudentListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Student>>>> {
    let data = svc::list(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Students retrieved", data)))
}

pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Student>>> {
    let data = svc::get(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Student retrieved", data)))
}

pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<ApiResponse<Student>>> {
    let data = svc::get_me(&state, &auth).await?;
    Ok(Json(ApiResponse::ok("Student profile retrieved", data)))
}

pub async fn create(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<CreateStudentRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Student>>)> {
    let data = svc::create(&state, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Student created", data)),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateStudentRequest>,
) -> AppResult<Json<ApiResponse<Student>>> {
    let data = svc::update(&state, id, req).await?;
    Ok(Json(ApiResponse::ok("Student updated", data)))
}

pub async fn remove(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate(&state, id).await?;
    Ok(Json(ApiResponse::ok("Student deactivated", ())))
}
