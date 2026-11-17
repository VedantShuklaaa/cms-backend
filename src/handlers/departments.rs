use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        auth::AuthUser,
        department::{
            CreateDepartmentRequest, Department, DepartmentListQuery, UpdateDepartmentRequest,
        },
    },
    services::departments as svc,
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
    ApiQuery(q): ApiQuery<DepartmentListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Department>>>> {
    let data = svc::list(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Departments retrieved", data)))
}

pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Department>>> {
    let data = svc::get(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Department retrieved", data)))
}

pub async fn create(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<CreateDepartmentRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Department>>)> {
    let data = svc::create(&state, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Department created", data)),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateDepartmentRequest>,
) -> AppResult<Json<ApiResponse<Department>>> {
    let data = svc::update(&state, id, req).await?;
    Ok(Json(ApiResponse::ok("Department updated", data)))
}

pub async fn remove(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate(&state, id).await?;
    Ok(Json(ApiResponse::ok("Department deactivated", ())))
}
