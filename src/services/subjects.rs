use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult, conflict_on_unique},
    middleware::rbac::ensure_role,
    models::{
        auth::AuthUser,
        subject::{
            AssignFacultyRequest, CreateSubjectRequest, MySubjectQuery, Subject, SubjectFaculty,
            SubjectListQuery, UpdateSubjectRequest,
        },
        user::Role,
    },
    services::{lookups, students as student_svc},
    state::AppState,
    utils::{
        query::{Paginated, Pagination, like_pattern},
        validation::{clean_code, clean_optional_text, clean_text},
    },
};

const SELECT: &str = "SELECT s.id, s.course_id, c.name AS course_name, c.department_id,
    d.name AS department_name, s.name, s.code, s.semester, s.credits, s.description,
    s.is_active, s.created_at, s.updated_at
    FROM subjects s
    JOIN courses c ON c.id = s.course_id
    JOIN departments d ON d.id = c.department_id";
const DUPLICATE: &str = "A subject with this code, or this name in this course, already exists";

fn validate_credits(credits: i16) -> AppResult<i16> {
    if (1..=10).contains(&credits) {
        Ok(credits)
    } else {
        Err(AppError::Validation(
            "credits must be between 1 and 10".into(),
        ))
    }
}

fn validate_semester(semester: i16, duration: i16) -> AppResult<i16> {
    if (1..=duration).contains(&semester) {
        Ok(semester)
    } else {
        Err(AppError::Validation(format!(
            "semester must be between 1 and {duration} for this course"
        )))
    }
}

async fn fetch(pool: &PgPool, id: Uuid) -> AppResult<Subject> {
    sqlx::query_as::<_, Subject>(&format!("{SELECT} WHERE s.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Subject not found".into()))
}

pub async fn list(
    state: &AppState,
    auth: &AuthUser,
    q: &SubjectListQuery,
) -> AppResult<Paginated<Subject>> {
    let p = Pagination::new(q.page, q.per_page);
    let include_inactive = q.include_inactive.unwrap_or(false) && auth.role == Role::Admin;
    let search = like_pattern(q.search.as_deref());
    const FILTER: &str = "WHERE ($1::bool OR s.is_active)
        AND ($2::uuid IS NULL OR s.course_id = $2)
        AND ($3::uuid IS NULL OR c.department_id = $3)
        AND ($4::smallint IS NULL OR s.semester = $4)
        AND ($5::text IS NULL OR s.name ILIKE $5 OR s.code ILIKE $5)
        AND ($6::uuid IS NULL OR EXISTS (
            SELECT 1 FROM faculty_subjects fs
            WHERE fs.faculty_id = $6 AND fs.subject_id = s.id
        ))";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM subjects s
        JOIN courses c ON c.id = s.course_id {FILTER}"
    ))
    .bind(include_inactive)
    .bind(q.course_id)
    .bind(q.department_id)
    .bind(q.semester)
    .bind(&search)
    .bind(q.faculty_id)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, Subject>(&format!(
        "{SELECT} {FILTER} ORDER BY s.semester, s.name LIMIT $7 OFFSET $8"
    ))
    .bind(include_inactive)
    .bind(q.course_id)
    .bind(q.department_id)
    .bind(q.semester)
    .bind(&search)
    .bind(q.faculty_id)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

/// Subjects tied to the caller: assigned subjects for faculty, the current
/// semester's subjects for students.
pub async fn mine(
    state: &AppState,
    auth: &AuthUser,
    q: &MySubjectQuery,
) -> AppResult<Paginated<Subject>> {
    ensure_role(auth, &[Role::Faculty, Role::Student])?;
    let p = Pagination::new(q.page, q.per_page);
    let search = like_pattern(q.search.as_deref());

    match auth.role {
        Role::Faculty => {
            let faculty_id = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
            const FILTER: &str = "WHERE s.is_active
                AND fs.faculty_id = $1
                AND ($2::text IS NULL OR s.name ILIKE $2 OR s.code ILIKE $2)";

            let total: i64 = sqlx::query_scalar(&format!(
                "SELECT COUNT(*) FROM subjects s
                 JOIN faculty_subjects fs ON fs.subject_id = s.id {FILTER}"
            ))
            .bind(faculty_id)
            .bind(&search)
            .fetch_one(&state.pool)
            .await?;

            let items = sqlx::query_as::<_, Subject>(&format!(
                "{SELECT} JOIN faculty_subjects fs ON fs.subject_id = s.id
                 {FILTER} ORDER BY s.semester, s.name LIMIT $3 OFFSET $4"
            ))
            .bind(faculty_id)
            .bind(&search)
            .bind(p.limit())
            .bind(p.offset())
            .fetch_all(&state.pool)
            .await?;

            Ok(Paginated::new(items, total, p))
        }
        Role::Student => {
            let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
            const FILTER: &str = "WHERE s.is_active
                AND s.course_id = $1
                AND s.semester = $2
                AND ($3::text IS NULL OR s.name ILIKE $3 OR s.code ILIKE $3)";

            let total: i64 =
                sqlx::query_scalar(&format!("SELECT COUNT(*) FROM subjects s {FILTER}"))
                    .bind(student.course_id)
                    .bind(student.current_semester)
                    .bind(&search)
                    .fetch_one(&state.pool)
                    .await?;

            let items = sqlx::query_as::<_, Subject>(&format!(
                "{SELECT} {FILTER} ORDER BY s.name LIMIT $4 OFFSET $5"
            ))
            .bind(student.course_id)
            .bind(student.current_semester)
            .bind(&search)
            .bind(p.limit())
            .bind(p.offset())
            .fetch_all(&state.pool)
            .await?;

            Ok(Paginated::new(items, total, p))
        }
        // ensure_role above guarantees this is unreachable.
        Role::Admin => unreachable!("ensure_role rejects admins"),
    }
}

pub async fn get(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Subject> {
    let subject = fetch(&state.pool, id).await?;
    if !subject.is_active && auth.role != Role::Admin {
        return Err(AppError::NotFound("Subject not found".into()));
    }
    Ok(subject)
}

pub async fn create(state: &AppState, req: CreateSubjectRequest) -> AppResult<Subject> {
    let name = clean_text("name", &req.name, 150)?;
    let code = clean_code("code", &req.code)?;
    let credits = validate_credits(req.credits)?;
    let description = clean_optional_text("description", req.description.as_deref(), 1000)?;
    let duration = lookups::active_course_duration(&state.pool, req.course_id).await?;
    let semester = validate_semester(req.semester, duration)?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO subjects (course_id, name, code, semester, credits, description)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(req.course_id)
    .bind(name)
    .bind(code)
    .bind(semester)
    .bind(credits)
    .bind(description)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;

    fetch(&state.pool, id).await
}

pub async fn update(state: &AppState, id: Uuid, req: UpdateSubjectRequest) -> AppResult<Subject> {
    let existing = fetch(&state.pool, id).await?;
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
    let credits = req.credits.map(validate_credits).transpose()?;
    let description = clean_optional_text("description", req.description.as_deref(), 1000)?;

    let semester = if req.course_id.is_some() || req.semester.is_some() {
        let course_id = req.course_id.unwrap_or(existing.course_id);
        let duration = lookups::active_course_duration(&state.pool, course_id).await?;
        Some(validate_semester(
            req.semester.unwrap_or(existing.semester),
            duration,
        )?)
    } else {
        None
    };

    let updated: Option<Uuid> = sqlx::query_scalar(
        "UPDATE subjects SET
            course_id = COALESCE($2, course_id),
            name = COALESCE($3, name),
            code = COALESCE($4, code),
            semester = COALESCE($5, semester),
            credits = COALESCE($6, credits),
            description = COALESCE($7, description),
            is_active = COALESCE($8, is_active)
         WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .bind(req.course_id)
    .bind(name)
    .bind(code)
    .bind(semester)
    .bind(credits)
    .bind(description)
    .bind(req.is_active)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;

    updated.ok_or_else(|| AppError::NotFound("Subject not found".into()))?;
    fetch(&state.pool, id).await
}

/// Soft delete.
pub async fn deactivate(state: &AppState, id: Uuid) -> AppResult<()> {
    let res = sqlx::query("UPDATE subjects SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Subject not found".into()));
    }
    Ok(())
}

const FACULTY_SELECT: &str = "SELECT fs.subject_id, fs.faculty_id, f.employee_id,
    f.first_name, f.last_name, u.email, f.department_id, d.name AS department_name,
    fs.assigned_at
    FROM faculty_subjects fs
    JOIN faculty f ON f.id = fs.faculty_id
    JOIN users u ON u.id = f.user_id
    JOIN departments d ON d.id = f.department_id";

pub async fn list_faculty(state: &AppState, subject_id: Uuid) -> AppResult<Vec<SubjectFaculty>> {
    fetch(&state.pool, subject_id).await?;
    Ok(sqlx::query_as::<_, SubjectFaculty>(&format!(
        "{FACULTY_SELECT} WHERE fs.subject_id = $1 ORDER BY f.first_name, f.last_name"
    ))
    .bind(subject_id)
    .fetch_all(&state.pool)
    .await?)
}

pub async fn assign_faculty(
    state: &AppState,
    subject_id: Uuid,
    req: AssignFacultyRequest,
) -> AppResult<SubjectFaculty> {
    let subject = fetch(&state.pool, subject_id).await?;
    if !subject.is_active {
        return Err(AppError::Validation("Subject is inactive".into()));
    }
    lookups::ensure_active_faculty(&state.pool, req.faculty_id).await?;

    sqlx::query(
        "INSERT INTO faculty_subjects (faculty_id, subject_id) VALUES ($1, $2)
         ON CONFLICT (faculty_id, subject_id) DO NOTHING",
    )
    .bind(req.faculty_id)
    .bind(subject_id)
    .execute(&state.pool)
    .await?;

    sqlx::query_as::<_, SubjectFaculty>(&format!(
        "{FACULTY_SELECT} WHERE fs.subject_id = $1 AND fs.faculty_id = $2"
    ))
    .bind(subject_id)
    .bind(req.faculty_id)
    .fetch_one(&state.pool)
    .await
    .map_err(Into::into)
}

pub async fn unassign_faculty(
    state: &AppState,
    subject_id: Uuid,
    faculty_id: Uuid,
) -> AppResult<()> {
    let res = sqlx::query("DELETE FROM faculty_subjects WHERE subject_id = $1 AND faculty_id = $2")
        .bind(subject_id)
        .bind(faculty_id)
        .execute(&state.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Faculty assignment not found".into()));
    }
    Ok(())
}
