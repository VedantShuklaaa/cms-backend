use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::{
        auth::AuthUser,
        notice::{AUDIENCES, CreateNoticeRequest, Notice, NoticeListQuery, UpdateNoticeRequest},
        user::Role,
    },
    state::AppState,
    utils::{
        query::{Paginated, Pagination, like_pattern},
        validation::{clean_choice, clean_optional_choice, clean_text},
    },
};

const SELECT: &str = "SELECT n.id, n.title, n.body, n.audience, n.created_by,
    u.email AS created_by_email, n.published_at, n.expires_at, n.is_active,
    n.created_at, n.updated_at
    FROM notices n JOIN users u ON u.id = n.created_by";

fn role_audience(role: Role) -> &'static str {
    match role {
        Role::Admin => "ADMIN",
        Role::Faculty => "FACULTY",
        Role::Student => "STUDENTS",
    }
}

fn ensure_expiry(
    published_at: chrono::DateTime<Utc>,
    expires_at: Option<chrono::DateTime<Utc>>,
) -> AppResult<()> {
    if let Some(expires) = expires_at
        && expires <= published_at
    {
        return Err(AppError::Validation(
            "expires_at must be after published_at".into(),
        ));
    }
    Ok(())
}

async fn fetch(pool: &PgPool, id: Uuid) -> AppResult<Notice> {
    sqlx::query_as::<_, Notice>(&format!("{SELECT} WHERE n.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Notice not found".into()))
}

fn is_visible(notice: &Notice, role: Role) -> bool {
    let now = Utc::now();
    notice.is_active
        && notice.published_at <= now
        && notice.expires_at.is_none_or(|expires| expires > now)
        && (notice.audience == "ALL" || notice.audience == role_audience(role))
}

pub async fn list(
    state: &AppState,
    auth: &AuthUser,
    q: &NoticeListQuery,
) -> AppResult<Paginated<Notice>> {
    let p = Pagination::new(q.page, q.per_page);
    let is_admin = auth.role == Role::Admin;
    let include_inactive = q.include_inactive.unwrap_or(false) && is_admin;
    let include_expired = q.include_expired.unwrap_or(false) && is_admin;
    let audience = q
        .audience
        .as_deref()
        .map(|a| clean_choice("audience", a, AUDIENCES))
        .transpose()?;
    let search = like_pattern(q.search.as_deref());
    const FILTER: &str = "WHERE ($1::bool OR n.is_active)
        AND ($2::bool OR n.published_at <= now())
        AND ($3::bool OR n.expires_at IS NULL OR n.expires_at > now())
        AND ($4::text IS NULL OR n.audience = $4)
        AND ($5::bool OR n.audience IN ('ALL', $6))
        AND ($7::text IS NULL OR n.title ILIKE $7 OR n.body ILIKE $7)";

    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM notices n {FILTER}"))
        .bind(include_inactive)
        .bind(is_admin)
        .bind(include_expired)
        .bind(&audience)
        .bind(is_admin)
        .bind(role_audience(auth.role))
        .bind(&search)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, Notice>(&format!(
        "{SELECT} {FILTER} ORDER BY n.published_at DESC LIMIT $8 OFFSET $9"
    ))
    .bind(include_inactive)
    .bind(is_admin)
    .bind(include_expired)
    .bind(&audience)
    .bind(is_admin)
    .bind(role_audience(auth.role))
    .bind(&search)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn get(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Notice> {
    let notice = fetch(&state.pool, id).await?;
    if auth.role != Role::Admin && !is_visible(&notice, auth.role) {
        return Err(AppError::NotFound("Notice not found".into()));
    }
    Ok(notice)
}

pub async fn create(
    state: &AppState,
    auth: &AuthUser,
    req: CreateNoticeRequest,
) -> AppResult<Notice> {
    let title = clean_text("title", &req.title, 200)?;
    let body = clean_text("body", &req.body, 5000)?;
    let audience = match req.audience.as_deref() {
        Some(value) => clean_choice("audience", value, AUDIENCES)?,
        None => "ALL".to_string(),
    };
    let published_at = req.published_at.unwrap_or_else(Utc::now);
    ensure_expiry(published_at, req.expires_at)?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO notices (title, body, audience, created_by, published_at, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(title)
    .bind(body)
    .bind(audience)
    .bind(auth.id)
    .bind(published_at)
    .bind(req.expires_at)
    .fetch_one(&state.pool)
    .await?;

    fetch(&state.pool, id).await
}

pub async fn update(state: &AppState, id: Uuid, req: UpdateNoticeRequest) -> AppResult<Notice> {
    let existing = fetch(&state.pool, id).await?;
    let title = req
        .title
        .as_deref()
        .map(|v| clean_text("title", v, 200))
        .transpose()?;
    let body = req
        .body
        .as_deref()
        .map(|v| clean_text("body", v, 5000))
        .transpose()?;
    let audience = clean_optional_choice("audience", req.audience.as_deref(), AUDIENCES)?;
    let published_at = req.published_at.unwrap_or(existing.published_at);
    let expires_at = req.expires_at.or(existing.expires_at);
    ensure_expiry(published_at, expires_at)?;

    sqlx::query(
        "UPDATE notices SET
            title = COALESCE($2, title),
            body = COALESCE($3, body),
            audience = COALESCE($4, audience),
            published_at = COALESCE($5, published_at),
            expires_at = COALESCE($6, expires_at),
            is_active = COALESCE($7, is_active)
         WHERE id = $1",
    )
    .bind(id)
    .bind(title)
    .bind(body)
    .bind(audience)
    .bind(req.published_at)
    .bind(req.expires_at)
    .bind(req.is_active)
    .execute(&state.pool)
    .await?;

    fetch(&state.pool, id).await
}

/// Soft delete.
pub async fn deactivate(state: &AppState, id: Uuid) -> AppResult<()> {
    let res = sqlx::query("UPDATE notices SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Notice not found".into()));
    }
    Ok(())
}
