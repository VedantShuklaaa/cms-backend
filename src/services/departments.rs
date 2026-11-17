use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult, conflict_on_unique},
    models::{
        auth::AuthUser,
        department::{
            CreateDepartmentRequest, Department, DepartmentListQuery, UpdateDepartmentRequest,
        },
        user::Role,
    },
    state::AppState,
    utils::{
        query::{Paginated, Pagination, like_pattern},
        validation::{clean_code, clean_optional_text, clean_text},
    },
};

const COLS: &str = "id, name, code, description, is_active, created_at, updated_at";
const DUPLICATE: &str = "A department with this name or code already exists";

async fn fetch(pool: &PgPool, id: Uuid) -> AppResult<Department> {
    sqlx::query_as::<_, Department>(&format!("SELECT {COLS} FROM departments WHERE id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Department not found".into()))
}

async fn ensure_no_active_courses(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let has: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM courses WHERE department_id = $1 AND is_active)",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    if has {
        return Err(AppError::Conflict(
            "Department still has active courses; deactivate or move them first".into(),
        ));
    }
    Ok(())
}

pub async fn list(
    state: &AppState,
    auth: &AuthUser,
    q: &DepartmentListQuery,
) -> AppResult<Paginated<Department>> {
    let p = Pagination::new(q.page, q.per_page);
    let include_inactive = q.include_inactive.unwrap_or(false) && auth.role == Role::Admin;
    let search = like_pattern(q.search.as_deref());
    const FILTER: &str =
        "WHERE ($1::bool OR is_active) AND ($2::text IS NULL OR name ILIKE $2 OR code ILIKE $2)";

    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM departments {FILTER}"))
        .bind(include_inactive)
        .bind(&search)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, Department>(&format!(
        "SELECT {COLS} FROM departments {FILTER} ORDER BY name LIMIT $3 OFFSET $4"
    ))
    .bind(include_inactive)
    .bind(&search)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn get(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Department> {
    let d = fetch(&state.pool, id).await?;
    if !d.is_active && auth.role != Role::Admin {
        return Err(AppError::NotFound("Department not found".into()));
    }
    Ok(d)
}

pub async fn create(state: &AppState, req: CreateDepartmentRequest) -> AppResult<Department> {
    let name = clean_text("name", &req.name, 100)?;
    let code = clean_code("code", &req.code)?;
    let description = clean_optional_text("description", req.description.as_deref(), 500)?;

    sqlx::query_as::<_, Department>(&format!(
        "INSERT INTO departments (name, code, description) VALUES ($1, $2, $3) RETURNING {COLS}"
    ))
    .bind(name)
    .bind(code)
    .bind(description)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))
}

pub async fn update(
    state: &AppState,
    id: Uuid,
    req: UpdateDepartmentRequest,
) -> AppResult<Department> {
    let name = req
        .name
        .as_deref()
        .map(|v| clean_text("name", v, 100))
        .transpose()?;
    let code = req
        .code
        .as_deref()
        .map(|v| clean_code("code", v))
        .transpose()?;
    let description = clean_optional_text("description", req.description.as_deref(), 500)?;

    if req.is_active == Some(false) {
        ensure_no_active_courses(&state.pool, id).await?;
    }

    sqlx::query_as::<_, Department>(&format!(
        "UPDATE departments SET
            name = COALESCE($2, name),
            code = COALESCE($3, code),
            description = COALESCE($4, description),
            is_active = COALESCE($5, is_active)
         WHERE id = $1 RETURNING {COLS}"
    ))
    .bind(id)
    .bind(name)
    .bind(code)
    .bind(description)
    .bind(req.is_active)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?
    .ok_or_else(|| AppError::NotFound("Department not found".into()))
}

/// Soft delete.
pub async fn deactivate(state: &AppState, id: Uuid) -> AppResult<()> {
    ensure_no_active_courses(&state.pool, id).await?;
    let res = sqlx::query("UPDATE departments SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Department not found".into()));
    }
    Ok(())
}
