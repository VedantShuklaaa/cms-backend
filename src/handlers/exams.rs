use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        auth::AuthUser,
        exam::{
            CreateExamRequest, CreateResultRequest, Exam, ExamListQuery, MyResultQuery, Result,
            ResultListQuery, UpdateExamRequest, UpdateResultRequest,
        },
    },
    services::exams as svc,
    state::AppState,
    utils::{
        extract::{ApiJson, ApiPath, ApiQuery},
        query::Paginated,
        response::ApiResponse,
    },
};

pub async fn list_exams(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<ExamListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Exam>>>> {
    let data = svc::list_exams(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Exams retrieved", data)))
}

pub async fn get_exam(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Exam>>> {
    let data = svc::get_exam(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Exam retrieved", data)))
}

pub async fn create_exam(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<CreateExamRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Exam>>)> {
    let data = svc::create_exam(&state, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Exam created", data)),
    ))
}

pub async fn update_exam(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateExamRequest>,
) -> AppResult<Json<ApiResponse<Exam>>> {
    let data = svc::update_exam(&state, id, req).await?;
    Ok(Json(ApiResponse::ok("Exam updated", data)))
}

pub async fn remove_exam(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate_exam(&state, id).await?;
    Ok(Json(ApiResponse::ok("Exam deactivated", ())))
}

pub async fn list_results(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<ResultListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Result>>>> {
    let data = svc::list_results(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Results retrieved", data)))
}

pub async fn my_results(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<MyResultQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Result>>>> {
    let data = svc::my_results(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Your results retrieved", data)))
}

pub async fn get_result(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Result>>> {
    let data = svc::get_result(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Result retrieved", data)))
}

pub async fn create_result(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiJson(req): ApiJson<CreateResultRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Result>>)> {
    let data = svc::create_result(&state, &auth, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Result recorded", data)),
    ))
}

pub async fn update_result(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateResultRequest>,
) -> AppResult<Json<ApiResponse<Result>>> {
    let data = svc::update_result(&state, &auth, id, req).await?;
    Ok(Json(ApiResponse::ok("Result updated", data)))
}

pub async fn remove_result(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate_result(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Result deactivated", ())))
}
