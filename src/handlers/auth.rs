use axum::{Json, extract::State};

use crate::{
    errors::AppResult,
    models::{
        auth::{AuthUser, ChangePasswordRequest, LoginRequest, LoginResponse},
        user::UserResponse,
    },
    services::auth::{self as auth_service, LoginResult},
    state::AppState,
    utils::{extract::ApiJson, response::ApiResponse},
};

fn login_response(r: LoginResult) -> LoginResponse {
    LoginResponse {
        access_token: r.token,
        token_type: "Bearer",
        expires_at: r.expires_at,
        user: UserResponse::from(r.user),
    }
}

pub async fn login(
    State(state): State<AppState>,
    ApiJson(req): ApiJson<LoginRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    let result = auth_service::login(&state, &req.email, &req.password).await?;
    Ok(Json(ApiResponse::ok(
        "Login successful",
        login_response(result),
    )))
}

pub async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<ApiResponse<()>>> {
    auth_service::logout(&state, &auth).await?;
    Ok(Json(ApiResponse::ok("Logged out successfully", ())))
}

pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let user = auth_service::get_user(&state, auth.id).await?;
    Ok(Json(ApiResponse::ok(
        "Current user",
        UserResponse::from(user),
    )))
}

pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    ApiJson(req): ApiJson<ChangePasswordRequest>,
) -> AppResult<Json<ApiResponse<LoginResponse>>> {
    let result =
        auth_service::change_password(&state, &auth, &req.current_password, &req.new_password)
            .await?;
    Ok(Json(ApiResponse::ok(
        "Password changed successfully",
        login_response(result),
    )))
}

/// Temporary RBAC probe. Remove in Phase 2.
pub async fn admin_ping(auth: AuthUser) -> Json<ApiResponse<()>> {
    tracing::debug!(user = %auth.id, "admin ping");
    Json(ApiResponse::ok("Admin access confirmed", ()))
}
