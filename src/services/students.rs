use chrono::{Datelike, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult, conflict_on_unique},
    middleware::rbac::ensure_role,
    models::{
        auth::AuthUser,
        student::{CreateStudentRequest, Student, StudentListQuery, UpdateStudentRequest},
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

const SELECT: &str = "SELECT s.id, s.user_id, u.email, s.course_id, c.name AS course_name,
    c.department_id, d.name AS department_name, s.enrollment_no, s.first_name, s.last_name,
    s.phone, s.date_of_birth, s.current_semester, s.admission_year, u.is_active,
    s.created_at, s.updated_at";
const FROM: &str = "FROM students s
    JOIN users u ON u.id = s.user_id
    JOIN courses c ON c.id = s.course_id
    JOIN departments d ON d.id = c.department_id";
const DUPLICATE: &str = "A student with this enrollment_no already exists";

fn validate_semester(sem: i16, duration: i16) -> AppResult<()> {
    if (1..=duration).contains(&sem) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "current_semester must be between 1 and {duration} for this course"
        )))
    }
}

fn validate_admission_year(year: i16, current_year: i32) -> AppResult<i16> {
    if i32::from(year) >= 1950 && i32::from(year) <= current_year + 1 {
        Ok(year)
    } else {
        Err(AppError::Validation(
            "admission_year is out of range".into(),
        ))
    }
}

fn validate_dob(dob: NaiveDate) -> AppResult<NaiveDate> {
    if dob < Utc::now().date_naive() {
        Ok(dob)
    } else {
        Err(AppError::Validation(
            "date_of_birth must be in the past".into(),
        ))
    }
}

async fn fetch(pool: &PgPool, id: Uuid) -> AppResult<Student> {
    sqlx::query_as::<_, Student>(&format!("{SELECT} {FROM} WHERE s.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Student not found".into()))
}

pub async fn fetch_by_user(pool: &PgPool, user_id: Uuid) -> AppResult<Student> {
    sqlx::query_as::<_, Student>(&format!("{SELECT} {FROM} WHERE s.user_id = $1"))
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Student profile not found".into()))
}

pub async fn list(
    state: &AppState,
    auth: &AuthUser,
    q: &StudentListQuery,
) -> AppResult<Paginated<Student>> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let p = Pagination::new(q.page, q.per_page);
    let include_inactive = q.include_inactive.unwrap_or(false) && auth.role == Role::Admin;
    let search = like_pattern(q.search.as_deref());
    const FILTER: &str = "WHERE ($1::bool OR u.is_active)
        AND ($2::uuid IS NULL OR s.course_id = $2)
        AND ($3::uuid IS NULL OR c.department_id = $3)
        AND ($4::smallint IS NULL OR s.current_semester = $4)
        AND ($5::text IS NULL OR s.first_name ILIKE $5 OR s.last_name ILIKE $5
             OR s.enrollment_no ILIKE $5 OR u.email ILIKE $5)";

    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) {FROM} {FILTER}"))
        .bind(include_inactive)
        .bind(q.course_id)
        .bind(q.department_id)
        .bind(q.semester)
        .bind(&search)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, Student>(&format!(
        "{SELECT} {FROM} {FILTER} ORDER BY s.enrollment_no LIMIT $6 OFFSET $7"
    ))
    .bind(include_inactive)
    .bind(q.course_id)
    .bind(q.department_id)
    .bind(q.semester)
    .bind(&search)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

/// Admin/faculty: any student. Student: only their own record.
pub async fn get(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Student> {
    let s = fetch(&state.pool, id).await?;
    if auth.role == Role::Student && s.user_id != auth.id {
        return Err(AppError::Forbidden(
            "You can only view your own student record".into(),
        ));
    }
    Ok(s)
}

pub async fn get_me(state: &AppState, auth: &AuthUser) -> AppResult<Student> {
    ensure_role(auth, &[Role::Student])?;
    fetch_by_user(&state.pool, auth.id).await
}

pub async fn create(state: &AppState, req: CreateStudentRequest) -> AppResult<Student> {
    let enrollment_no = clean_code("enrollment_no", &req.enrollment_no)?;
    let first_name = clean_text("first_name", &req.first_name, 100)?;
    let last_name = clean_text("last_name", &req.last_name, 100)?;
    let phone = req
        .phone
        .as_deref()
        .map(|p| clean_phone("phone", p))
        .transpose()?;
    let dob = req.date_of_birth.map(validate_dob).transpose()?;
    let year = validate_admission_year(req.admission_year, Utc::now().year())?;
    let duration = lookups::active_course_duration(&state.pool, req.course_id).await?;
    validate_semester(req.current_semester, duration)?;
    let account = NewAccount::prepare(&req.email, &req.password).await?;

    let mut tx = state.pool.begin().await?;
    let user_id = users::insert_account(&mut tx, &account, Role::Student).await?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO students (user_id, course_id, enrollment_no, first_name, last_name,
                               phone, date_of_birth, current_semester, admission_year)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
    )
    .bind(user_id)
    .bind(req.course_id)
    .bind(enrollment_no)
    .bind(first_name)
    .bind(last_name)
    .bind(phone)
    .bind(dob)
    .bind(req.current_semester)
    .bind(year)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;
    tx.commit().await?;

    fetch(&state.pool, id).await
}

pub async fn update(state: &AppState, id: Uuid, req: UpdateStudentRequest) -> AppResult<Student> {
    let existing = fetch(&state.pool, id).await?;

    let enrollment_no = req
        .enrollment_no
        .as_deref()
        .map(|v| clean_code("enrollment_no", v))
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
    let phone = req
        .phone
        .as_deref()
        .map(|p| clean_phone("phone", p))
        .transpose()?;
    let dob = req.date_of_birth.map(validate_dob).transpose()?;
    let year = req
        .admission_year
        .map(|y| validate_admission_year(y, Utc::now().year()))
        .transpose()?;

    // Course and semester must stay consistent with each other.
    if req.course_id.is_some() || req.current_semester.is_some() {
        let course_id = req.course_id.unwrap_or(existing.course_id);
        let semester = req.current_semester.unwrap_or(existing.current_semester);
        let duration = lookups::active_course_duration(&state.pool, course_id).await?;
        validate_semester(semester, duration)?;
    }

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "UPDATE students SET
            course_id = COALESCE($2, course_id),
            enrollment_no = COALESCE($3, enrollment_no),
            first_name = COALESCE($4, first_name),
            last_name = COALESCE($5, last_name),
            phone = COALESCE($6, phone),
            date_of_birth = COALESCE($7, date_of_birth),
            current_semester = COALESCE($8, current_semester),
            admission_year = COALESCE($9, admission_year)
         WHERE id = $1",
    )
    .bind(id)
    .bind(req.course_id)
    .bind(enrollment_no)
    .bind(first_name)
    .bind(last_name)
    .bind(phone)
    .bind(dob)
    .bind(req.current_semester)
    .bind(year)
    .execute(&mut *tx)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;

    if let Some(active) = req.is_active {
        users::set_active(&mut *tx, existing.user_id, active).await?;
    }
    tx.commit().await?;

    fetch(&state.pool, id).await
}

/// Soft delete: the account is deactivated, history stays intact.
pub async fn deactivate(state: &AppState, id: Uuid) -> AppResult<()> {
    let s = fetch(&state.pool, id).await?;
    users::set_active(&state.pool, s.user_id, false).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semester_must_fit_course_duration() {
        assert!(validate_semester(1, 8).is_ok());
        assert!(validate_semester(8, 8).is_ok());
        assert!(validate_semester(0, 8).is_err());
        assert!(validate_semester(9, 8).is_err());
    }

    #[test]
    fn admission_year_bounds() {
        assert!(validate_admission_year(2024, 2026).is_ok());
        assert!(validate_admission_year(2027, 2026).is_ok());
        assert!(validate_admission_year(2028, 2026).is_err());
        assert!(validate_admission_year(1949, 2026).is_err());
    }

    #[test]
    fn dob_must_be_past() {
        let today = Utc::now().date_naive();
        assert!(validate_dob(today).is_err());
        assert!(validate_dob(today.pred_opt().unwrap()).is_ok());
    }
}
