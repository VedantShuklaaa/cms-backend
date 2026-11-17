use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        attendance::{
            Attendance, AttendanceListQuery, CreateAttendanceRequest, MyAttendanceQuery,
            UpdateAttendanceRequest,
        },
        auth::AuthUser,
    },
    services::attendance as svc,
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
    ApiQuery(q): ApiQuery<AttendanceListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Attendance>>>> {
    let data = svc::list(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Attendance retrieved", data)))
}

pub async fn mine(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<MyAttendanceQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Attendance>>>> {
    let data = svc::mine(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Your attendance retrieved", data)))
}

pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Attendance>>> {
    let data = svc::get(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Attendance record retrieved", data)))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiJson(req): ApiJson<CreateAttendanceRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Attendance>>)> {
    let data = svc::create(&state, &auth, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Attendance recorded", data)),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateAttendanceRequest>,
) -> AppResult<Json<ApiResponse<Attendance>>> {
    let data = svc::update(&state, &auth, id, req).await?;
    Ok(Json(ApiResponse::ok("Attendance updated", data)))
}

pub async fn remove(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Attendance record deactivated", ())))
}
