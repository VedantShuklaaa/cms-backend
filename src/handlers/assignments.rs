use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        assignment::{
            Assignment, AssignmentListQuery, CreateAssignmentRequest, CreateSubmissionRequest,
            GradeSubmissionRequest, MySubmissionListQuery, Submission, SubmissionListQuery,
            UpdateAssignmentRequest,
        },
        auth::AuthUser,
    },
    services::assignments as svc,
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
    ApiQuery(q): ApiQuery<AssignmentListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Assignment>>>> {
    let data = svc::list(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Assignments retrieved", data)))
}

pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Assignment>>> {
    let data = svc::get(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Assignment retrieved", data)))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiJson(req): ApiJson<CreateAssignmentRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Assignment>>)> {
    let data = svc::create(&state, &auth, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Assignment created", data)),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateAssignmentRequest>,
) -> AppResult<Json<ApiResponse<Assignment>>> {
    let data = svc::update(&state, &auth, id, req).await?;
    Ok(Json(ApiResponse::ok("Assignment updated", data)))
}

pub async fn remove(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Assignment deactivated", ())))
}

pub async fn submit(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<CreateSubmissionRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Submission>>)> {
    let data = svc::submit(&state, &auth, id, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Assignment submitted", data)),
    ))
}

pub async fn list_submissions(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiQuery(q): ApiQuery<SubmissionListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Submission>>>> {
    let data = svc::list_submissions(&state, &auth, id, &q).await?;
    Ok(Json(ApiResponse::ok("Submissions retrieved", data)))
}

pub async fn grade_submission(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath((assignment_id, submission_id)): ApiPath<(Uuid, Uuid)>,
    ApiJson(req): ApiJson<GradeSubmissionRequest>,
) -> AppResult<Json<ApiResponse<Submission>>> {
    let data = svc::grade_submission(&state, &auth, assignment_id, submission_id, req).await?;
    Ok(Json(ApiResponse::ok("Submission graded", data)))
}

pub async fn remove_submission(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath((assignment_id, submission_id)): ApiPath<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate_submission(&state, &auth, assignment_id, submission_id).await?;
    Ok(Json(ApiResponse::ok("Submission deactivated", ())))
}

pub async fn my_submissions(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<MySubmissionListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Submission>>>> {
    let data = svc::my_submissions(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Your submissions retrieved", data)))
}
