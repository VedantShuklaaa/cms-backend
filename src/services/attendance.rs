use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult, conflict_on_unique},
    middleware::rbac::ensure_role,
    models::{
        attendance::{
            ATTENDANCE_STATUSES, Attendance, AttendanceListQuery, CreateAttendanceRequest,
            MyAttendanceQuery, UpdateAttendanceRequest,
        },
        auth::AuthUser,
        user::Role,
    },
    services::{lookups, students as student_svc},
    state::AppState,
    utils::{
        query::{Paginated, Pagination},
        validation::clean_choice,
    },
};

const SELECT: &str = "SELECT a.id, a.subject_id, s.name AS subject_name, s.code AS subject_code,
    a.student_id, st.enrollment_no, st.first_name AS student_first_name,
    st.last_name AS student_last_name, a.marked_by, a.attendance_date, a.status,
    a.is_active, a.created_at, a.updated_at
    FROM attendance a
    JOIN subjects s ON s.id = a.subject_id
    JOIN students st ON st.id = a.student_id";
const DUPLICATE: &str = "Attendance for this student, subject and date already exists";

fn ensure_date_range(from: Option<NaiveDate>, to: Option<NaiveDate>) -> AppResult<()> {
    if let (Some(f), Some(t)) = (from, to)
        && f > t
    {
        return Err(AppError::Validation(
            "date_from must be on or before date_to".into(),
        ));
    }
    Ok(())
}

fn ensure_not_future(date: NaiveDate) -> AppResult<NaiveDate> {
    if date <= Utc::now().date_naive() {
        Ok(date)
    } else {
        Err(AppError::Validation(
            "attendance_date cannot be in the future".into(),
        ))
    }
}

async fn fetch(pool: &PgPool, id: Uuid) -> AppResult<Attendance> {
    sqlx::query_as::<_, Attendance>(&format!("{SELECT} WHERE a.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Attendance record not found".into()))
}

async fn ensure_can_manage(
    state: &AppState,
    auth: &AuthUser,
    subject_id: Uuid,
) -> AppResult<Option<Uuid>> {
    match auth.role {
        Role::Admin => Ok(None),
        Role::Faculty => {
            let faculty_id = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
            lookups::ensure_faculty_assigned(&state.pool, faculty_id, subject_id).await?;
            Ok(Some(faculty_id))
        }
        Role::Student => Err(AppError::Forbidden(
            "Students cannot manage attendance".into(),
        )),
    }
}

pub async fn list(
    state: &AppState,
    auth: &AuthUser,
    q: &AttendanceListQuery,
) -> AppResult<Paginated<Attendance>> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    ensure_date_range(q.date_from, q.date_to)?;
    let status = q
        .status
        .as_deref()
        .map(|s| clean_choice("status", s, ATTENDANCE_STATUSES))
        .transpose()?;
    let p = Pagination::new(q.page, q.per_page);
    let is_admin = auth.role == Role::Admin;
    let include_inactive = q.include_inactive.unwrap_or(false) && is_admin;
    let faculty_id = if is_admin {
        Uuid::nil()
    } else {
        lookups::faculty_id_for_user(&state.pool, auth.id).await?
    };

    const FILTER: &str = "WHERE ($1::bool OR a.is_active)
        AND ($2::uuid IS NULL OR a.subject_id = $2)
        AND ($3::uuid IS NULL OR a.student_id = $3)
        AND ($4::text IS NULL OR a.status = $4)
        AND ($5::date IS NULL OR a.attendance_date >= $5)
        AND ($6::date IS NULL OR a.attendance_date <= $6)
        AND ($7::bool OR EXISTS (
            SELECT 1 FROM faculty_subjects fs
            WHERE fs.faculty_id = $8 AND fs.subject_id = a.subject_id
        ))";

    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM attendance a {FILTER}"))
        .bind(include_inactive)
        .bind(q.subject_id)
        .bind(q.student_id)
        .bind(&status)
        .bind(q.date_from)
        .bind(q.date_to)
        .bind(is_admin)
        .bind(faculty_id)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, Attendance>(&format!(
        "{SELECT} {FILTER}
         ORDER BY a.attendance_date DESC, st.enrollment_no LIMIT $9 OFFSET $10"
    ))
    .bind(include_inactive)
    .bind(q.subject_id)
    .bind(q.student_id)
    .bind(&status)
    .bind(q.date_from)
    .bind(q.date_to)
    .bind(is_admin)
    .bind(faculty_id)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn mine(
    state: &AppState,
    auth: &AuthUser,
    q: &MyAttendanceQuery,
) -> AppResult<Paginated<Attendance>> {
    ensure_role(auth, &[Role::Student])?;
    ensure_date_range(q.date_from, q.date_to)?;
    let status = q
        .status
        .as_deref()
        .map(|s| clean_choice("status", s, ATTENDANCE_STATUSES))
        .transpose()?;
    let p = Pagination::new(q.page, q.per_page);
    const FILTER: &str = "WHERE st.user_id = $1 AND a.is_active
        AND ($2::uuid IS NULL OR a.subject_id = $2)
        AND ($3::text IS NULL OR a.status = $3)
        AND ($4::date IS NULL OR a.attendance_date >= $4)
        AND ($5::date IS NULL OR a.attendance_date <= $5)";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM attendance a
         JOIN students st ON st.id = a.student_id {FILTER}"
    ))
    .bind(auth.id)
    .bind(q.subject_id)
    .bind(&status)
    .bind(q.date_from)
    .bind(q.date_to)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, Attendance>(&format!(
        "{SELECT} {FILTER}
         ORDER BY a.attendance_date DESC, s.name LIMIT $6 OFFSET $7"
    ))
    .bind(auth.id)
    .bind(q.subject_id)
    .bind(&status)
    .bind(q.date_from)
    .bind(q.date_to)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn get(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Attendance> {
    let record = fetch(&state.pool, id).await?;
    match auth.role {
        Role::Admin => Ok(record),
        Role::Faculty => {
            let faculty_id = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
            lookups::ensure_faculty_assigned(&state.pool, faculty_id, record.subject_id).await?;
            if !record.is_active {
                return Err(AppError::NotFound("Attendance record not found".into()));
            }
            Ok(record)
        }
        Role::Student => {
            let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
            if student.id != record.student_id {
                return Err(AppError::Forbidden(
                    "You can only view your own attendance".into(),
                ));
            }
            if !record.is_active {
                return Err(AppError::NotFound("Attendance record not found".into()));
            }
            Ok(record)
        }
    }
}

pub async fn create(
    state: &AppState,
    auth: &AuthUser,
    req: CreateAttendanceRequest,
) -> AppResult<Attendance> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let status = clean_choice("status", &req.status, ATTENDANCE_STATUSES)?;
    let date = ensure_not_future(req.attendance_date)?;
    let subject_course = lookups::active_subject_course(&state.pool, req.subject_id).await?;
    let student_course = lookups::active_student_course(&state.pool, req.student_id).await?;
    if subject_course != student_course {
        return Err(AppError::Validation(
            "Student is not enrolled in the subject's course".into(),
        ));
    }
    let marked_by = ensure_can_manage(state, auth, req.subject_id).await?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO attendance (subject_id, student_id, marked_by, attendance_date, status)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (subject_id, student_id, attendance_date)
         DO UPDATE SET status = EXCLUDED.status, marked_by = EXCLUDED.marked_by,
                       is_active = TRUE, updated_at = now()
         RETURNING id",
    )
    .bind(req.subject_id)
    .bind(req.student_id)
    .bind(marked_by)
    .bind(date)
    .bind(status)
    .fetch_one(&state.pool)
    .await?;

    fetch(&state.pool, id).await
}

pub async fn update(
    state: &AppState,
    auth: &AuthUser,
    id: Uuid,
    req: UpdateAttendanceRequest,
) -> AppResult<Attendance> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let record = fetch(&state.pool, id).await?;
    ensure_can_manage(state, auth, record.subject_id).await?;

    let status = req
        .status
        .as_deref()
        .map(|s| clean_choice("status", s, ATTENDANCE_STATUSES))
        .transpose()?;
    let date = req.attendance_date.map(ensure_not_future).transpose()?;

    sqlx::query(
        "UPDATE attendance SET
            attendance_date = COALESCE($2, attendance_date),
            status = COALESCE($3, status),
            is_active = COALESCE($4, is_active)
         WHERE id = $1",
    )
    .bind(id)
    .bind(date)
    .bind(status)
    .bind(req.is_active)
    .execute(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;

    fetch(&state.pool, id).await
}

/// Soft delete.
pub async fn deactivate(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<()> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let record = fetch(&state.pool, id).await?;
    ensure_can_manage(state, auth, record.subject_id).await?;
    sqlx::query("UPDATE attendance SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(())
}
