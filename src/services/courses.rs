use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult, conflict_on_unique},
    models::{
        auth::AuthUser,
        course::{Course, CourseListQuery, CreateCourseRequest, UpdateCourseRequest},
        user::Role,
    },
    state::AppState,
    utils::{
        query::{Paginated, Pagination, like_pattern},
        validation::{clean_code, clean_optional_text, clean_text},
    },
};

const SELECT: &str = "SELECT c.id, c.department_id, d.name AS department_name, c.name, c.code,
    c.duration_semesters, c.description, c.is_active, c.created_at, c.updated_at
    FROM courses c JOIN departments d ON d.id = c.department_id";
const DUPLICATE: &str = "A course with this code already exists";

fn validate_duration(v: i16) -> AppResult<i16> {
    if (1..=12).contains(&v) {
        Ok(v)
    } else {
        Err(AppError::Validation(
            "duration_semesters must be between 1 and 12".into(),
        ))
    }
}

async fn ensure_active_department(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let ok: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM departments WHERE id = $1 AND is_active)")
            .bind(id)
            .fetch_one(pool)
            .await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::Validation(
            "Department does not exist or is inactive".into(),
        ))
    }
}

async fn fetch(pool: &PgPool, id: Uuid) -> AppResult<Course> {
    sqlx::query_as::<_, Course>(&format!("{SELECT} WHERE c.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Course not found".into()))
}

pub async fn list(
    state: &AppState,
    auth: &AuthUser,
    q: &CourseListQuery,
) -> AppResult<Paginated<Course>> {
    let p = Pagination::new(q.page, q.per_page);
    let include_inactive = q.include_inactive.unwrap_or(false) && auth.role == Role::Admin;
    let search = like_pattern(q.search.as_deref());
    const FILTER: &str = "WHERE ($1::bool OR c.is_active)
        AND ($2::uuid IS NULL OR c.department_id = $2)
        AND ($3::text IS NULL OR c.name ILIKE $3 OR c.code ILIKE $3)";

    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM courses c {FILTER}"))
        .bind(include_inactive)
        .bind(q.department_id)
        .bind(&search)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, Course>(&format!(
        "{SELECT} {FILTER} ORDER BY c.name LIMIT $4 OFFSET $5"
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

pub async fn get(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Course> {
    let c = fetch(&state.pool, id).await?;
    if !c.is_active && auth.role != Role::Admin {
        return Err(AppError::NotFound("Course not found".into()));
    }
    Ok(c)
}

pub async fn create(state: &AppState, req: CreateCourseRequest) -> AppResult<Course> {
    let name = clean_text("name", &req.name, 150)?;
    let code = clean_code("code", &req.code)?;
    let duration = validate_duration(req.duration_semesters)?;
    let description = clean_optional_text("description", req.description.as_deref(), 1000)?;
    ensure_active_department(&state.pool, req.department_id).await?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO courses (department_id, name, code, duration_semesters, description)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(req.department_id)
    .bind(name)
    .bind(code)
    .bind(duration)
    .bind(description)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;

    fetch(&state.pool, id).await
}

pub async fn update(state: &AppState, id: Uuid, req: UpdateCourseRequest) -> AppResult<Course> {
    let name = req
        .name
        .as_deref()
        .map(|v| clean_text("name", v, 150))
        .transpose()?;
    let code = req
        .code
        .as_deref()
        .map(|v| clean_code("code", v))
        .transpose()?;
    let duration = req.duration_semesters.map(validate_duration).transpose()?;
    let description = clean_optional_text("description", req.description.as_deref(), 1000)?;
    if let Some(dept) = req.department_id {
        ensure_active_department(&state.pool, dept).await?;
    }

    let updated: Option<Uuid> = sqlx::query_scalar(
        "UPDATE courses SET
            department_id = COALESCE($2, department_id),
            name = COALESCE($3, name),
            code = COALESCE($4, code),
            duration_semesters = COALESCE($5, duration_semesters),
            description = COALESCE($6, description),
            is_active = COALESCE($7, is_active)
         WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .bind(req.department_id)
    .bind(name)
    .bind(code)
    .bind(duration)
    .bind(description)
    .bind(req.is_active)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;

    updated.ok_or_else(|| AppError::NotFound("Course not found".into()))?;
    fetch(&state.pool, id).await
}

/// Soft delete.
pub async fn deactivate(state: &AppState, id: Uuid) -> AppResult<()> {
    let res = sqlx::query("UPDATE courses SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Course not found".into()));
    }
    Ok(())
}
