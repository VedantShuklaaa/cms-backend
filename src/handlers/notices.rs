use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        auth::AuthUser,
        notice::{CreateNoticeRequest, Notice, NoticeListQuery, UpdateNoticeRequest},
    },
    services::notices as svc,
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
    ApiQuery(q): ApiQuery<NoticeListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Notice>>>> {
    let data = svc::list(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Notices retrieved", data)))
}

pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Notice>>> {
    let data = svc::get(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Notice retrieved", data)))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiJson(req): ApiJson<CreateNoticeRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Notice>>)> {
    let data = svc::create(&state, &auth, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Notice created", data)),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateNoticeRequest>,
) -> AppResult<Json<ApiResponse<Notice>>> {
    let data = svc::update(&state, id, req).await?;
    Ok(Json(ApiResponse::ok("Notice updated", data)))
}

pub async fn remove(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate(&state, id).await?;
    Ok(Json(ApiResponse::ok("Notice deactivated", ())))
}
