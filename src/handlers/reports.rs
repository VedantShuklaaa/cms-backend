use axum::{Json, extract::State};

use crate::{
    errors::AppResult,
    models::{
        auth::AuthUser,
        report::{
            AttendanceReportQuery, AttendanceReportRow, FeeReportQuery, FeeReportRow,
            OverviewReport, ResultsReportQuery, ResultsReportRow,
        },
    },
    services::reports as svc,
    state::AppState,
    utils::{extract::ApiQuery, response::ApiResponse},
};

pub async fn overview(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<ApiResponse<OverviewReport>>> {
    let data = svc::overview(&state, &auth).await?;
    Ok(Json(ApiResponse::ok("Overview report retrieved", data)))
}

pub async fn attendance(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<AttendanceReportQuery>,
) -> AppResult<Json<ApiResponse<Vec<AttendanceReportRow>>>> {
    let data = svc::attendance(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Attendance report retrieved", data)))
}

pub async fn results(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<ResultsReportQuery>,
) -> AppResult<Json<ApiResponse<Vec<ResultsReportRow>>>> {
    let data = svc::results(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Results report retrieved", data)))
}

pub async fn fees(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiQuery(q): ApiQuery<FeeReportQuery>,
) -> AppResult<Json<ApiResponse<Vec<FeeReportRow>>>> {
    let data = svc::fees(&state, &auth, &q).await?;
    Ok(Json(ApiResponse::ok("Fees report retrieved", data)))
}
