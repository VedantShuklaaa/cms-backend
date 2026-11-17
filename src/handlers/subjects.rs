use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        auth::AuthUser,
        subject::{
            AssignFacultyRequest, CreateSubjectRequest, MySubjectQuery, Subject, SubjectFaculty,
            SubjectListQuery, UpdateSubjectRequest,
        },
    },
    services::subjects as svc,
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
    ApiQuery(q): ApiQuery<SubjectListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Subject>>>> {
    let data = svc::list(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Subjects retrieved", data)))
}

pub async fn mine(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<MySubjectQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Subject>>>> {
    let data = svc::mine(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Your subjects retrieved", data)))
}

pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Subject>>> {
    let data = svc::get(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Subject retrieved", data)))
}

pub async fn create(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<CreateSubjectRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<Subject>>)> {
    let data = svc::create(&state, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Subject created", data)),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateSubjectRequest>,
) -> AppResult<Json<ApiResponse<Subject>>> {
    let data = svc::update(&state, id, req).await?;
    Ok(Json(ApiResponse::ok("Subject updated", data)))
}

pub async fn remove(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate(&state, id).await?;
    Ok(Json(ApiResponse::ok("Subject deactivated", ())))
}

pub async fn list_faculty(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<SubjectFaculty>>>> {
    let data = svc::list_faculty(&state, id).await?;
    Ok(Json(ApiResponse::ok("Subject faculty retrieved", data)))
}

pub async fn assign_faculty(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<AssignFacultyRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<SubjectFaculty>>)> {
    let data = svc::assign_faculty(&state, id, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Faculty assigned to subject", data)),
    ))
}

pub async fn unassign_faculty(
    State(state): State<AppState>,
    ApiPath((subject_id, faculty_id)): ApiPath<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::unassign_faculty(&state, subject_id, faculty_id).await?;
    Ok(Json(ApiResponse::ok("Faculty unassigned from subject", ())))
}
