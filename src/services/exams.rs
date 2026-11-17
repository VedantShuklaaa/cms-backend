use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult, conflict_on_unique},
    middleware::rbac::ensure_role,
    models::{
        auth::AuthUser,
        exam::{
            CreateExamRequest, CreateResultRequest, EXAM_TYPES, Exam, ExamListQuery, MyResultQuery,
            Result, ResultListQuery, UpdateExamRequest, UpdateResultRequest,
        },
        user::Role,
    },
    services::lookups,
    state::AppState,
    utils::{
        query::{Paginated, Pagination, like_pattern},
        validation::{clean_choice, clean_optional_text, clean_text},
    },
};

const EXAM_SELECT: &str = "SELECT e.id, e.subject_id, s.name AS subject_name,
    s.code AS subject_code, s.course_id, c.name AS course_name, e.name, e.exam_type,
    e.exam_date, e.max_marks, e.is_active, e.created_at, e.updated_at
    FROM exams e
    JOIN subjects s ON s.id = e.subject_id
    JOIN courses c ON c.id = s.course_id";
const EXAM_DUPLICATE: &str = "An exam with this name already exists for the subject";

const RESULT_SELECT: &str = "SELECT r.id, r.exam_id, e.name AS exam_name, e.exam_type,
    e.subject_id, s.name AS subject_name, r.student_id, st.user_id AS student_user_id,
    st.enrollment_no, st.first_name AS student_first_name,
    st.last_name AS student_last_name, r.marks_obtained, e.max_marks, r.grade,
    r.recorded_by, r.is_active, r.created_at, r.updated_at
    FROM results r
    JOIN exams e ON e.id = r.exam_id
    JOIN subjects s ON s.id = e.subject_id
    JOIN students st ON st.id = r.student_id";

fn validate_marks(marks: i16, max_marks: i16) -> AppResult<i16> {
    if (0..=max_marks).contains(&marks) {
        Ok(marks)
    } else {
        Err(AppError::Validation(format!(
            "marks_obtained must be between 0 and {max_marks}"
        )))
    }
}

fn validate_max_marks(max_marks: i16) -> AppResult<i16> {
    if (1..=1000).contains(&max_marks) {
        Ok(max_marks)
    } else {
        Err(AppError::Validation(
            "max_marks must be between 1 and 1000".into(),
        ))
    }
}

async fn fetch_exam(pool: &PgPool, id: Uuid) -> AppResult<Exam> {
    sqlx::query_as::<_, Exam>(&format!("{EXAM_SELECT} WHERE e.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Exam not found".into()))
}

async fn fetch_result(pool: &PgPool, id: Uuid) -> AppResult<Result> {
    sqlx::query_as::<_, Result>(&format!("{RESULT_SELECT} WHERE r.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Result not found".into()))
}

pub async fn list_exams(
    state: &AppState,
    auth: &AuthUser,
    q: &ExamListQuery,
) -> AppResult<Paginated<Exam>> {
    let p = Pagination::new(q.page, q.per_page);
    let include_inactive = q.include_inactive.unwrap_or(false) && auth.role == Role::Admin;
    let exam_type = q
        .exam_type
        .as_deref()
        .map(|t| clean_choice("exam_type", t, EXAM_TYPES))
        .transpose()?;
    let search = like_pattern(q.search.as_deref());
    const FILTER: &str = "WHERE ($1::bool OR e.is_active)
        AND ($2::uuid IS NULL OR e.subject_id = $2)
        AND ($3::uuid IS NULL OR s.course_id = $3)
        AND ($4::text IS NULL OR e.exam_type = $4)
        AND ($5::text IS NULL OR e.name ILIKE $5 OR s.name ILIKE $5 OR s.code ILIKE $5)";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM exams e JOIN subjects s ON s.id = e.subject_id {FILTER}"
    ))
    .bind(include_inactive)
    .bind(q.subject_id)
    .bind(q.course_id)
    .bind(&exam_type)
    .bind(&search)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, Exam>(&format!(
        "{EXAM_SELECT} {FILTER} ORDER BY e.exam_date DESC, e.name LIMIT $6 OFFSET $7"
    ))
    .bind(include_inactive)
    .bind(q.subject_id)
    .bind(q.course_id)
    .bind(&exam_type)
    .bind(&search)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn get_exam(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Exam> {
    let exam = fetch_exam(&state.pool, id).await?;
    if !exam.is_active && auth.role != Role::Admin {
        return Err(AppError::NotFound("Exam not found".into()));
    }
    Ok(exam)
}

pub async fn create_exam(state: &AppState, req: CreateExamRequest) -> AppResult<Exam> {
    let name = clean_text("name", &req.name, 100)?;
    let exam_type = clean_choice("exam_type", &req.exam_type, EXAM_TYPES)?;
    let max_marks = validate_max_marks(req.max_marks)?;
    lookups::active_subject_course(&state.pool, req.subject_id).await?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO exams (subject_id, name, exam_type, exam_date, max_marks)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(req.subject_id)
    .bind(name)
    .bind(exam_type)
    .bind(req.exam_date)
    .bind(max_marks)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, EXAM_DUPLICATE))?;

    fetch_exam(&state.pool, id).await
}

pub async fn update_exam(state: &AppState, id: Uuid, req: UpdateExamRequest) -> AppResult<Exam> {
    let existing = fetch_exam(&state.pool, id).await?;
    let name = req
        .name
        .as_deref()
        .map(|v| clean_text("name", v, 100))
        .transpose()?;
    let exam_type = req
        .exam_type
        .as_deref()
        .map(|t| clean_choice("exam_type", t, EXAM_TYPES))
        .transpose()?;
    let max_marks = req.max_marks.map(validate_max_marks).transpose()?;

    if let Some(subject_id) = req.subject_id {
        lookups::active_subject_course(&state.pool, subject_id).await?;
        if subject_id != existing.subject_id {
            let has_results: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM results WHERE exam_id = $1 AND is_active)",
            )
            .bind(id)
            .fetch_one(&state.pool)
            .await?;
            if has_results {
                return Err(AppError::Conflict(
                    "Exam already has results; its subject cannot change".into(),
                ));
            }
        }
    }

    if let Some(new_max) = max_marks {
        let over_max: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM results
                           WHERE exam_id = $1 AND is_active AND marks_obtained > $2)",
        )
        .bind(id)
        .bind(new_max)
        .fetch_one(&state.pool)
        .await?;
        if over_max {
            return Err(AppError::Validation(
                "max_marks is lower than an existing result".into(),
            ));
        }
    }

    let updated: Option<Uuid> = sqlx::query_scalar(
        "UPDATE exams SET
            subject_id = COALESCE($2, subject_id),
            name = COALESCE($3, name),
            exam_type = COALESCE($4, exam_type),
            exam_date = COALESCE($5, exam_date),
            max_marks = COALESCE($6, max_marks),
            is_active = COALESCE($7, is_active)
         WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .bind(req.subject_id)
    .bind(name)
    .bind(exam_type)
    .bind(req.exam_date)
    .bind(max_marks)
    .bind(req.is_active)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, EXAM_DUPLICATE))?;

    updated.ok_or_else(|| AppError::NotFound("Exam not found".into()))?;
    fetch_exam(&state.pool, id).await
}

/// Soft delete.
pub async fn deactivate_exam(state: &AppState, id: Uuid) -> AppResult<()> {
    let res = sqlx::query("UPDATE exams SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Exam not found".into()));
    }
    Ok(())
}

pub async fn list_results(
    state: &AppState,
    auth: &AuthUser,
    q: &ResultListQuery,
) -> AppResult<Paginated<Result>> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let p = Pagination::new(q.page, q.per_page);
    let is_admin = auth.role == Role::Admin;
    let include_inactive = q.include_inactive.unwrap_or(false) && is_admin;
    let faculty_id = if is_admin {
        Uuid::nil()
    } else {
        lookups::faculty_id_for_user(&state.pool, auth.id).await?
    };

    const FILTER: &str = "WHERE ($1::bool OR r.is_active)
        AND ($2::uuid IS NULL OR r.exam_id = $2)
        AND ($3::uuid IS NULL OR e.subject_id = $3)
        AND ($4::uuid IS NULL OR r.student_id = $4)
        AND ($5::bool OR EXISTS (
            SELECT 1 FROM faculty_subjects fs
            WHERE fs.faculty_id = $6 AND fs.subject_id = e.subject_id
        ))";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM results r JOIN exams e ON e.id = r.exam_id {FILTER}"
    ))
    .bind(include_inactive)
    .bind(q.exam_id)
    .bind(q.subject_id)
    .bind(q.student_id)
    .bind(is_admin)
    .bind(faculty_id)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, Result>(&format!(
        "{RESULT_SELECT} {FILTER}
         ORDER BY e.exam_date DESC, st.enrollment_no LIMIT $7 OFFSET $8"
    ))
    .bind(include_inactive)
    .bind(q.exam_id)
    .bind(q.subject_id)
    .bind(q.student_id)
    .bind(is_admin)
    .bind(faculty_id)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn my_results(
    state: &AppState,
    auth: &AuthUser,
    q: &MyResultQuery,
) -> AppResult<Paginated<Result>> {
    ensure_role(auth, &[Role::Student])?;
    let p = Pagination::new(q.page, q.per_page);
    const FILTER: &str = "WHERE st.user_id = $1 AND r.is_active
        AND ($2::uuid IS NULL OR e.subject_id = $2)
        AND ($3::uuid IS NULL OR r.exam_id = $3)";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM results r
         JOIN exams e ON e.id = r.exam_id
         JOIN students st ON st.id = r.student_id {FILTER}"
    ))
    .bind(auth.id)
    .bind(q.subject_id)
    .bind(q.exam_id)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, Result>(&format!(
        "{RESULT_SELECT} {FILTER}
         ORDER BY e.exam_date DESC, e.name LIMIT $4 OFFSET $5"
    ))
    .bind(auth.id)
    .bind(q.subject_id)
    .bind(q.exam_id)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn get_result(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Result> {
    let result = fetch_result(&state.pool, id).await?;
    match auth.role {
        Role::Admin => Ok(result),
        Role::Faculty => {
            let faculty_id = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
            lookups::ensure_faculty_assigned(&state.pool, faculty_id, result.subject_id).await?;
            if !result.is_active {
                return Err(AppError::NotFound("Result not found".into()));
            }
            Ok(result)
        }
        Role::Student => {
            if result.student_user_id != auth.id {
                return Err(AppError::Forbidden(
                    "You can only view your own results".into(),
                ));
            }
            if !result.is_active {
                return Err(AppError::NotFound("Result not found".into()));
            }
            Ok(result)
        }
    }
}

pub async fn create_result(
    state: &AppState,
    auth: &AuthUser,
    req: CreateResultRequest,
) -> AppResult<Result> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let (subject_id, max_marks) = lookups::active_exam_subject(&state.pool, req.exam_id).await?;
    let marks = validate_marks(req.marks_obtained, max_marks)?;
    let grade = clean_optional_text("grade", req.grade.as_deref(), 10)?;
    let subject_course = lookups::active_subject_course(&state.pool, subject_id).await?;
    let student_course = lookups::active_student_course(&state.pool, req.student_id).await?;
    if subject_course != student_course {
        return Err(AppError::Validation(
            "Student is not enrolled in the exam's course".into(),
        ));
    }

    let recorded_by = match auth.role {
        Role::Faculty => {
            let faculty_id = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
            lookups::ensure_faculty_assigned(&state.pool, faculty_id, subject_id).await?;
            Some(faculty_id)
        }
        Role::Admin => None,
        Role::Student => unreachable!("ensure_role rejects students"),
    };

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO results (exam_id, student_id, marks_obtained, grade, recorded_by)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (exam_id, student_id)
         DO UPDATE SET marks_obtained = EXCLUDED.marks_obtained, grade = EXCLUDED.grade,
                       recorded_by = EXCLUDED.recorded_by, is_active = TRUE, updated_at = now()
         RETURNING id",
    )
    .bind(req.exam_id)
    .bind(req.student_id)
    .bind(marks)
    .bind(grade)
    .bind(recorded_by)
    .fetch_one(&state.pool)
    .await?;

    fetch_result(&state.pool, id).await
}

pub async fn update_result(
    state: &AppState,
    auth: &AuthUser,
    id: Uuid,
    req: UpdateResultRequest,
) -> AppResult<Result> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let existing = fetch_result(&state.pool, id).await?;
    if auth.role == Role::Faculty {
        let faculty_id = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
        lookups::ensure_faculty_assigned(&state.pool, faculty_id, existing.subject_id).await?;
    }
    let marks = req
        .marks_obtained
        .map(|m| validate_marks(m, existing.max_marks))
        .transpose()?;
    let grade = clean_optional_text("grade", req.grade.as_deref(), 10)?;

    sqlx::query(
        "UPDATE results SET
            marks_obtained = COALESCE($2, marks_obtained),
            grade = COALESCE($3, grade),
            is_active = COALESCE($4, is_active)
         WHERE id = $1",
    )
    .bind(id)
    .bind(marks)
    .bind(grade)
    .bind(req.is_active)
    .execute(&state.pool)
    .await?;

    fetch_result(&state.pool, id).await
}

/// Soft delete.
pub async fn deactivate_result(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<()> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let existing = fetch_result(&state.pool, id).await?;
    if auth.role == Role::Faculty {
        let faculty_id = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
        lookups::ensure_faculty_assigned(&state.pool, faculty_id, existing.subject_id).await?;
    }
    sqlx::query("UPDATE results SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(())
}
