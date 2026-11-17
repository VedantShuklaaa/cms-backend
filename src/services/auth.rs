use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::{auth::AuthUser, user::User},
    state::AppState,
    utils::{jwt, password, validation},
};

const USER_COLUMNS: &str =
    "id, email, password_hash, role, is_active, created_at, updated_at, password_changed_at";
const INVALID_CREDENTIALS: &str = "Invalid email or password";

pub struct LoginResult {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: User,
}

async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
    let sql = format!("SELECT {USER_COLUMNS} FROM users WHERE lower(email) = $1");
    Ok(sqlx::query_as::<_, User>(&sql)
        .bind(email)
        .fetch_optional(pool)
        .await?)
}

async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
    let sql = format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1");
    Ok(sqlx::query_as::<_, User>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_user(state: &AppState, id: Uuid) -> AppResult<User> {
    find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))
}

pub async fn login(state: &AppState, email: &str, pw: &str) -> AppResult<LoginResult> {
    if pw.is_empty() || pw.len() > validation::MAX_PASSWORD_LEN * 4 {
        return Err(AppError::Unauthorized(INVALID_CREDENTIALS.into()));
    }

    let email = validation::normalize_email(email);
    let user = find_by_email(&state.pool, &email).await?;

    // Always run a verification so unknown emails cost the same as wrong passwords.
    let hash = match &user {
        Some(u) => u.password_hash.clone(),
        None => password::dummy_hash().to_string(),
    };
    let valid = password::verify(pw.to_string(), hash).await?;

    let user = match user {
        Some(u) if valid && u.is_active => u,
        _ => return Err(AppError::Unauthorized(INVALID_CREDENTIALS.into())),
    };

    let issued = jwt::issue_token(
        user.id,
        &state.config.jwt_secret,
        state.config.jwt_expiration_minutes,
    )?;
    Ok(LoginResult {
        token: issued.token,
        expires_at: issued.expires_at,
        user,
    })
}

/// Validates a bearer token and builds the caller context. Role comes from the DB, never the token.
pub async fn authenticate(state: &AppState, token: &str) -> AppResult<AuthUser> {
    let claims = jwt::decode_token(token, &state.config.jwt_secret)?;

    let revoked: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM revoked_tokens WHERE jti = $1)")
            .bind(claims.jti)
            .fetch_one(&state.pool)
            .await?;
    if revoked {
        return Err(AppError::Unauthorized("Token has been revoked".into()));
    }

    let user = find_by_id(&state.pool, claims.sub)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid token".into()))?;
    if !user.is_active {
        return Err(AppError::Unauthorized("Account is deactivated".into()));
    }
    if claims.iat < user.password_changed_at.timestamp() {
        return Err(AppError::Unauthorized(
            "Token is no longer valid, please log in again".into(),
        ));
    }

    let expires_at = DateTime::<Utc>::from_timestamp(claims.exp, 0)
        .ok_or_else(|| AppError::Unauthorized("Invalid token".into()))?;

    Ok(AuthUser {
        id: user.id,
        email: user.email,
        role: user.role,
        jti: claims.jti,
        expires_at,
    })
}

pub async fn logout(state: &AppState, auth: &AuthUser) -> AppResult<()> {
    // Rows past their expiry can no longer be replayed, so drop them.
    sqlx::query("DELETE FROM revoked_tokens WHERE expires_at < now()")
        .execute(&state.pool)
        .await?;
    sqlx::query(
        "INSERT INTO revoked_tokens (jti, user_id, expires_at) VALUES ($1, $2, $3)
         ON CONFLICT (jti) DO NOTHING",
    )
    .bind(auth.jti)
    .bind(auth.id)
    .bind(auth.expires_at)
    .execute(&state.pool)
    .await?;
    Ok(())
}

pub async fn change_password(
    state: &AppState,
    auth: &AuthUser,
    current: &str,
    new: &str,
) -> AppResult<LoginResult> {
    validation::validate_password(new)?;
    if current.len() > validation::MAX_PASSWORD_LEN * 4 {
        return Err(AppError::BadRequest("Current password is incorrect".into()));
    }
    if current == new {
        return Err(AppError::Validation(
            "New password must differ from the current password".into(),
        ));
    }

    let user = find_by_id(&state.pool, auth.id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Account not found".into()))?;

    // 400, not 401: the session is fine, only the supplied password is wrong.
    if !password::verify(current.to_string(), user.password_hash.clone()).await? {
        return Err(AppError::BadRequest("Current password is incorrect".into()));
    }

    let new_hash = password::hash(new.to_string()).await?;
    let changed_at = Utc::now();
    sqlx::query("UPDATE users SET password_hash = $1, password_changed_at = $2 WHERE id = $3")
        .bind(new_hash)
        .bind(changed_at)
        .bind(user.id)
        .execute(&state.pool)
        .await?;

    // Every token issued before changed_at is now rejected; hand back a fresh one.
    let issued = jwt::issue_token(
        user.id,
        &state.config.jwt_secret,
        state.config.jwt_expiration_minutes,
    )?;
    Ok(LoginResult {
        token: issued.token,
        expires_at: issued.expires_at,
        user,
    })
}

/// Creates the first admin only when no admin exists. Returns true if one was created.
pub async fn bootstrap_admin(pool: &PgPool, email: &str, pw: &str) -> AppResult<bool> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE role = 'ADMIN')")
            .fetch_one(pool)
            .await?;
    if exists {
        return Ok(false);
    }

    let email = validation::normalize_email(email);
    validation::validate_email(&email)?;
    validation::validate_password(pw)?;
    let hash = password::hash(pw.to_string()).await?;

    sqlx::query("INSERT INTO users (email, password_hash, role) VALUES ($1, $2, 'ADMIN')")
        .bind(email)
        .bind(hash)
        .execute(pool)
        .await?;
    Ok(true)
}
