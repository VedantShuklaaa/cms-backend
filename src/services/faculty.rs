use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult, conflict_on_unique},
    middleware::rbac::ensure_role,
    models::{
        auth::AuthUser,
        faculty::{CreateFacultyRequest, Faculty, FacultyListQuery, UpdateFacultyRequest},
        user::Role,
    },
    services::{
        lookups,
        users::{self, NewAccount},
    },
    state::AppState,
    utils::{
        query::{Paginated, Pagination, like_pattern},
        validation::{clean_code, clean_phone, clean_text},
    },
};

const SELECT: &str = "SELECT f.id, f.user_id, u.email, f.department_id, d.name AS department_name,
    f.employee_id, f.first_name, f.last_name, f.phone, f.designation, f.joined_on,
    u.is_active, f.created_at, f.updated_at";
const FROM: &str = "FROM faculty f
    JOIN users u ON u.id = f.user_id
    JOIN departments d ON d.id = f.department_id";
const DUPLICATE: &str = "A faculty member with this employee_id already exists";

async fn fetch(pool: &PgPool, id: Uuid) -> AppResult<Faculty> {
    sqlx::query_as::<_, Faculty>(&format!("{SELECT} {FROM} WHERE f.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Faculty member not found".into()))
}

async fn fetch_by_user(pool: &PgPool, user_id: Uuid) -> AppResult<Faculty> {
    sqlx::query_as::<_, Faculty>(&format!("{SELECT} {FROM} WHERE f.user_id = $1"))
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Faculty profile not found".into()))
}

pub async fn list(
    state: &AppState,
    auth: &AuthUser,
    q: &FacultyListQuery,
) -> AppResult<Paginated<Faculty>> {
    ensure_role(auth, &[Role::Admin])?;
    let p = Pagination::new(q.page, q.per_page);
    let include_inactive = q.include_inactive.unwrap_or(false);
    let search = like_pattern(q.search.as_deref());
    const FILTER: &str = "WHERE ($1::bool OR u.is_active)
        AND ($2::uuid IS NULL OR f.department_id = $2)
        AND ($3::text IS NULL OR f.first_name ILIKE $3 OR f.last_name ILIKE $3
             OR f.employee_id ILIKE $3 OR u.email ILIKE $3)";

    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) {FROM} {FILTER}"))
        .bind(include_inactive)
        .bind(q.department_id)
        .bind(&search)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, Faculty>(&format!(
        "{SELECT} {FROM} {FILTER} ORDER BY f.first_name, f.last_name LIMIT $4 OFFSET $5"
    ))
    .bind(include_inactive)
    .bind(q.department_id)
    .bind(&search)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

/// Admin: anyone. Faculty: only their own record. Students: never.
pub async fn get(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Faculty> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let f = fetch(&state.pool, id).await?;
    if auth.role == Role::Faculty && f.user_id != auth.id {
        return Err(AppError::Forbidden(
            "You can only view your own faculty record".into(),
        ));
    }
    Ok(f)
}

pub async fn get_me(state: &AppState, auth: &AuthUser) -> AppResult<Faculty> {
    ensure_role(auth, &[Role::Faculty])?;
    fetch_by_user(&state.pool, auth.id).await
}

pub async fn create(state: &AppState, req: CreateFacultyRequest) -> AppResult<Faculty> {
    let employee_id = clean_code("employee_id", &req.employee_id)?;
    let first_name = clean_text("first_name", &req.first_name, 100)?;
    let last_name = clean_text("last_name", &req.last_name, 100)?;
    let designation = clean_text("designation", &req.designation, 100)?;
    let phone = clean_optional_phone(req.phone.as_deref())?;
    lookups::ensure_active_department(&state.pool, req.department_id).await?;
    let account = NewAccount::prepare(&req.email, &req.password).await?;

    let mut tx = state.pool.begin().await?;
    let user_id = users::insert_account(&mut tx, &account, Role::Faculty).await?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO faculty (user_id, department_id, employee_id, first_name, last_name,
                              phone, designation, joined_on)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(user_id)
    .bind(req.department_id)
    .bind(employee_id)
    .bind(first_name)
    .bind(last_name)
    .bind(phone)
    .bind(designation)
    .bind(req.joined_on)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;
    tx.commit().await?;

    fetch(&state.pool, id).await
}

pub async fn update(state: &AppState, id: Uuid, req: UpdateFacultyRequest) -> AppResult<Faculty> {
    let employee_id = req
        .employee_id
        .as_deref()
        .map(|v| clean_code("employee_id", v))
        .transpose()?;
    let first_name = req
        .first_name
        .as_deref()
        .map(|v| clean_text("first_name", v, 100))
        .transpose()?;
    let last_name = req
        .last_name
        .as_deref()
        .map(|v| clean_text("last_name", v, 100))
        .transpose()?;
    let designation = req
        .designation
        .as_deref()
        .map(|v| clean_text("designation", v, 100))
        .transpose()?;
    let phone = clean_optional_phone(req.phone.as_deref())?;
    if let Some(dept) = req.department_id {
        lookups::ensure_active_department(&state.pool, dept).await?;
    }

    let mut tx = state.pool.begin().await?;
    let user_id: Uuid = sqlx::query_scalar(
        "UPDATE faculty SET
            department_id = COALESCE($2, department_id),
            employee_id = COALESCE($3, employee_id),
            first_name = COALESCE($4, first_name),
            last_name = COALESCE($5, last_name),
            phone = COALESCE($6, phone),
            designation = COALESCE($7, designation),
            joined_on = COALESCE($8, joined_on)
         WHERE id = $1 RETURNING user_id",
    )
    .bind(id)
    .bind(req.department_id)
    .bind(employee_id)
    .bind(first_name)
    .bind(last_name)
    .bind(phone)
    .bind(designation)
    .bind(req.joined_on)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?
    .ok_or_else(|| AppError::NotFound("Faculty member not found".into()))?;

    if let Some(active) = req.is_active {
        users::set_active(&mut *tx, user_id, active).await?;
    }
    tx.commit().await?;

    fetch(&state.pool, id).await
}

/// Soft delete: the account is deactivated, the profile row stays for academic history.
pub async fn deactivate(state: &AppState, id: Uuid) -> AppResult<()> {
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM faculty WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Faculty member not found".into()))?;
    users::set_active(&state.pool, user_id, false).await
}

fn clean_optional_phone(raw: Option<&str>) -> AppResult<Option<String>> {
    raw.map(|p| clean_phone("phone", p)).transpose()
}
