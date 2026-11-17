use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        auth::AuthUser,
        fee::{
            CreateFeePaymentRequest, CreateFeeStructureRequest, FeePayment, FeePaymentListQuery,
            FeeStatement, FeeStatementListQuery, FeeStructure, FeeStructureListQuery,
            MyPaymentListQuery, UpdateFeeStructureRequest,
        },
    },
    services::fees as svc,
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
    ApiQuery(q): ApiQuery<FeeStructureListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<FeeStructure>>>> {
    let data = svc::list(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Fee structures retrieved", data)))
}

pub async fn get_one(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<FeeStructure>>> {
    let data = svc::get(&state, &auth, id).await?;
    Ok(Json(ApiResponse::ok("Fee structure retrieved", data)))
}

pub async fn create(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<CreateFeeStructureRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<FeeStructure>>)> {
    let data = svc::create(&state, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Fee structure created", data)),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<UpdateFeeStructureRequest>,
) -> AppResult<Json<ApiResponse<FeeStructure>>> {
    let data = svc::update(&state, id, req).await?;
    Ok(Json(ApiResponse::ok("Fee structure updated", data)))
}

pub async fn remove(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate(&state, id).await?;
    Ok(Json(ApiResponse::ok("Fee structure deactivated", ())))
}

pub async fn list_payments(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiQuery(q): ApiQuery<FeePaymentListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<FeePayment>>>> {
    let data = svc::list_payments(&state, &auth, id, &q).await?;
    Ok(Json(ApiResponse::ok("Fee payments retrieved", data)))
}

pub async fn create_payment(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(req): ApiJson<CreateFeePaymentRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<FeePayment>>)> {
    let data = svc::create_payment(&state, &auth, id, req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::ok("Fee payment recorded", data)),
    ))
}

pub async fn remove_payment(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiPath((fee_structure_id, payment_id)): ApiPath<(Uuid, Uuid)>,
) -> AppResult<Json<ApiResponse<()>>> {
    svc::deactivate_payment(&state, &auth, fee_structure_id, payment_id).await?;
    Ok(Json(ApiResponse::ok("Fee payment deactivated", ())))
}

pub async fn my_statements(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<FeeStatementListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<FeeStatement>>>> {
    let data = svc::my_statements(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Your fee statements retrieved", data)))
}

pub async fn my_payments(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<MyPaymentListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<FeePayment>>>> {
    let data = svc::my_payments(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Your fee payments retrieved", data)))
}
