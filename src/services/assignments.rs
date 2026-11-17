use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult, conflict_on_unique},
    middleware::rbac::ensure_role,
    models::{
        assignment::{
            Assignment, AssignmentListQuery, CreateAssignmentRequest, CreateSubmissionRequest,
            GradeSubmissionRequest, MySubmissionListQuery, Submission, SubmissionListQuery,
            UpdateAssignmentRequest,
        },
        auth::AuthUser,
        user::Role,
    },
    services::{lookups, students as student_svc},
    state::AppState,
    utils::{
        query::{Paginated, Pagination, like_pattern},
        validation::clean_text,
    },
};

const SELECT: &str = "SELECT a.id, a.subject_id, s.name AS subject_name,
    s.code AS subject_code, s.course_id, a.faculty_id,
    f.first_name AS faculty_first_name, f.last_name AS faculty_last_name, a.title,
    a.description, a.due_date, a.max_marks, a.is_active, a.created_at, a.updated_at
    FROM assignments a
    JOIN subjects s ON s.id = a.subject_id
    JOIN faculty f ON f.id = a.faculty_id";
const DUPLICATE: &str = "An assignment with this title already exists for the subject";

const SUBMISSION_SELECT: &str = "SELECT sub.id, sub.assignment_id,
    a.title AS assignment_title, sub.student_id, st.user_id AS student_user_id,
    st.enrollment_no, st.first_name AS student_first_name,
    st.last_name AS student_last_name, sub.content, sub.submitted_at,
    sub.marks_obtained, sub.graded_by, sub.graded_at, sub.is_active,
    sub.created_at, sub.updated_at
    FROM assignment_submissions sub
    JOIN assignments a ON a.id = sub.assignment_id
    JOIN students st ON st.id = sub.student_id";

fn validate_max_marks(max_marks: Option<i16>) -> AppResult<Option<i16>> {
    match max_marks {
        Some(value) if !(1..=1000).contains(&value) => Err(AppError::Validation(
            "max_marks must be between 1 and 1000".into(),
        )),
        other => Ok(other),
    }
}

async fn fetch(pool: &PgPool, id: Uuid) -> AppResult<Assignment> {
    sqlx::query_as::<_, Assignment>(&format!("{SELECT} WHERE a.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Assignment not found".into()))
}

async fn fetch_submission(pool: &PgPool, id: Uuid) -> AppResult<Submission> {
    sqlx::query_as::<_, Submission>(&format!("{SUBMISSION_SELECT} WHERE sub.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Submission not found".into()))
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
            "Students cannot manage assignments".into(),
        )),
    }
}

pub async fn list(
    state: &AppState,
    auth: &AuthUser,
    q: &AssignmentListQuery,
) -> AppResult<Paginated<Assignment>> {
    let p = Pagination::new(q.page, q.per_page);
    let is_admin = auth.role == Role::Admin;
    let include_inactive = q.include_inactive.unwrap_or(false) && is_admin;
    let search = like_pattern(q.search.as_deref());

    let (course_scope, course_id, faculty_bypass, faculty_id) = match auth.role {
        Role::Admin => (true, Uuid::nil(), true, Uuid::nil()),
        Role::Faculty => (
            true,
            Uuid::nil(),
            false,
            lookups::faculty_id_for_user(&state.pool, auth.id).await?,
        ),
        Role::Student => {
            let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
            (false, student.course_id, true, Uuid::nil())
        }
    };

    const FILTER: &str = "WHERE ($1::bool OR a.is_active)
        AND ($2::uuid IS NULL OR a.subject_id = $2)
        AND ($3::uuid IS NULL OR s.course_id = $3)
        AND ($4::text IS NULL OR a.title ILIKE $4 OR a.description ILIKE $4
             OR s.name ILIKE $4 OR s.code ILIKE $4)
        AND ($5::bool OR s.course_id = $6)
        AND ($7::bool OR EXISTS (
            SELECT 1 FROM faculty_subjects fs
            WHERE fs.faculty_id = $8 AND fs.subject_id = a.subject_id
        ))";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM assignments a
         JOIN subjects s ON s.id = a.subject_id {FILTER}"
    ))
    .bind(include_inactive)
    .bind(q.subject_id)
    .bind(q.course_id)
    .bind(&search)
    .bind(course_scope)
    .bind(course_id)
    .bind(faculty_bypass)
    .bind(faculty_id)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, Assignment>(&format!(
        "{SELECT} {FILTER} ORDER BY a.due_date DESC, a.title LIMIT $9 OFFSET $10"
    ))
    .bind(include_inactive)
    .bind(q.subject_id)
    .bind(q.course_id)
    .bind(&search)
    .bind(course_scope)
    .bind(course_id)
    .bind(faculty_bypass)
    .bind(faculty_id)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn get(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<Assignment> {
    let assignment = fetch(&state.pool, id).await?;
    match auth.role {
        Role::Admin => Ok(assignment),
        Role::Faculty => {
            let faculty_id = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
            lookups::ensure_faculty_assigned(&state.pool, faculty_id, assignment.subject_id)
                .await?;
            if !assignment.is_active {
                return Err(AppError::NotFound("Assignment not found".into()));
            }
            Ok(assignment)
        }
        Role::Student => {
            let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
            if assignment.course_id != student.course_id {
                return Err(AppError::Forbidden(
                    "You can only view assignments for your own course".into(),
                ));
            }
            if !assignment.is_active {
                return Err(AppError::NotFound("Assignment not found".into()));
            }
            Ok(assignment)
        }
    }
}

pub async fn create(
    state: &AppState,
    auth: &AuthUser,
    req: CreateAssignmentRequest,
) -> AppResult<Assignment> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let title = clean_text("title", &req.title, 150)?;
    let description = clean_text("description", &req.description, 5000)?;
    let max_marks = validate_max_marks(req.max_marks)?;
    lookups::active_subject_course(&state.pool, req.subject_id).await?;

    let faculty_id = match auth.role {
        Role::Faculty => {
            let own = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
            lookups::ensure_faculty_assigned(&state.pool, own, req.subject_id).await?;
            own
        }
        Role::Admin => {
            let chosen = req.faculty_id.ok_or_else(|| {
                AppError::Validation(
                    "faculty_id is required when an admin creates an assignment".into(),
                )
            })?;
            lookups::ensure_active_faculty(&state.pool, chosen).await?;
            lookups::ensure_faculty_assigned(&state.pool, chosen, req.subject_id).await?;
            chosen
        }
        Role::Student => unreachable!("ensure_role rejects students"),
    };

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO assignments (subject_id, faculty_id, title, description, due_date, max_marks)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(req.subject_id)
    .bind(faculty_id)
    .bind(title)
    .bind(description)
    .bind(req.due_date)
    .bind(max_marks)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;

    fetch(&state.pool, id).await
}

pub async fn update(
    state: &AppState,
    auth: &AuthUser,
    id: Uuid,
    req: UpdateAssignmentRequest,
) -> AppResult<Assignment> {
    let existing = fetch(&state.pool, id).await?;
    ensure_can_manage(state, auth, existing.subject_id).await?;
    let title = req
        .title
        .as_deref()
        .map(|v| clean_text("title", v, 150))
        .transpose()?;
    let description = req
        .description
        .as_deref()
        .map(|v| clean_text("description", v, 5000))
        .transpose()?;
    let max_marks = validate_max_marks(req.max_marks)?;

    if let Some(new_max) = max_marks {
        let over: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM assignment_submissions
                           WHERE assignment_id = $1 AND is_active
                             AND marks_obtained > $2)",
        )
        .bind(id)
        .bind(new_max)
        .fetch_one(&state.pool)
        .await?;
        if over {
            return Err(AppError::Validation(
                "max_marks is lower than an existing submission score".into(),
            ));
        }
    }

    let updated: Option<Uuid> = sqlx::query_scalar(
        "UPDATE assignments SET
            title = COALESCE($2, title),
            description = COALESCE($3, description),
            due_date = COALESCE($4, due_date),
            max_marks = COALESCE($5, max_marks),
            is_active = COALESCE($6, is_active)
         WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(req.due_date)
    .bind(max_marks)
    .bind(req.is_active)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, DUPLICATE))?;

    updated.ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;
    fetch(&state.pool, id).await
}

/// Soft delete.
pub async fn deactivate(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<()> {
    let existing = fetch(&state.pool, id).await?;
    ensure_can_manage(state, auth, existing.subject_id).await?;
    sqlx::query("UPDATE assignments SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(())
}

pub async fn submit(
    state: &AppState,
    auth: &AuthUser,
    assignment_id: Uuid,
    req: CreateSubmissionRequest,
) -> AppResult<Submission> {
    ensure_role(auth, &[Role::Student])?;
    let assignment = fetch(&state.pool, assignment_id).await?;
    if !assignment.is_active {
        return Err(AppError::NotFound("Assignment not found".into()));
    }
    let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
    if assignment.course_id != student.course_id {
        return Err(AppError::Forbidden(
            "You can only submit assignments for your own course".into(),
        ));
    }
    if Utc::now().date_naive() > assignment.due_date {
        return Err(AppError::Validation(
            "The assignment deadline has passed".into(),
        ));
    }
    let content = clean_text("content", &req.content, 10000)?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO assignment_submissions (assignment_id, student_id, content)
         VALUES ($1, $2, $3)
         ON CONFLICT (assignment_id, student_id)
         DO UPDATE SET content = EXCLUDED.content, submitted_at = now(),
                       marks_obtained = NULL, graded_by = NULL, graded_at = NULL,
                       is_active = TRUE, updated_at = now()
         RETURNING id",
    )
    .bind(assignment_id)
    .bind(student.id)
    .bind(content)
    .fetch_one(&state.pool)
    .await?;

    fetch_submission(&state.pool, id).await
}

pub async fn list_submissions(
    state: &AppState,
    auth: &AuthUser,
    assignment_id: Uuid,
    q: &SubmissionListQuery,
) -> AppResult<Paginated<Submission>> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let assignment = fetch(&state.pool, assignment_id).await?;
    if auth.role == Role::Faculty {
        let faculty_id = lookups::faculty_id_for_user(&state.pool, auth.id).await?;
        lookups::ensure_faculty_assigned(&state.pool, faculty_id, assignment.subject_id).await?;
    }
    let p = Pagination::new(q.page, q.per_page);
    let include_inactive = q.include_inactive.unwrap_or(false) && auth.role == Role::Admin;
    const FILTER: &str = "WHERE sub.assignment_id = $1
        AND ($2::bool OR sub.is_active)
        AND ($3::uuid IS NULL OR sub.student_id = $3)";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM assignment_submissions sub {FILTER}"
    ))
    .bind(assignment_id)
    .bind(include_inactive)
    .bind(q.student_id)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, Submission>(&format!(
        "{SUBMISSION_SELECT} {FILTER}
         ORDER BY st.enrollment_no LIMIT $4 OFFSET $5"
    ))
    .bind(assignment_id)
    .bind(include_inactive)
    .bind(q.student_id)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn grade_submission(
    state: &AppState,
    auth: &AuthUser,
    assignment_id: Uuid,
    submission_id: Uuid,
    req: GradeSubmissionRequest,
) -> AppResult<Submission> {
    let assignment = fetch(&state.pool, assignment_id).await?;
    let grader = ensure_can_manage(state, auth, assignment.subject_id).await?;
    let submission = fetch_submission(&state.pool, submission_id).await?;
    if submission.assignment_id != assignment_id {
        return Err(AppError::NotFound("Submission not found".into()));
    }

    if let Some(marks) = req.marks_obtained {
        let max = assignment.max_marks.ok_or_else(|| {
            AppError::Validation("This assignment has no max_marks configured".into())
        })?;
        if !(0..=max).contains(&marks) {
            return Err(AppError::Validation(format!(
                "marks_obtained must be between 0 and {max}"
            )));
        }
    }

    sqlx::query(
        "UPDATE assignment_submissions SET
            marks_obtained = COALESCE($2, marks_obtained),
            graded_by = CASE WHEN $2::smallint IS NULL THEN graded_by ELSE $3::uuid END,
            graded_at = CASE WHEN $2::smallint IS NULL THEN graded_at ELSE now() END,
            is_active = COALESCE($4, is_active)
         WHERE id = $1",
    )
    .bind(submission_id)
    .bind(req.marks_obtained)
    .bind(grader)
    .bind(req.is_active)
    .execute(&state.pool)
    .await?;

    fetch_submission(&state.pool, submission_id).await
}

/// Soft delete.
pub async fn deactivate_submission(
    state: &AppState,
    auth: &AuthUser,
    assignment_id: Uuid,
    submission_id: Uuid,
) -> AppResult<()> {
    let assignment = fetch(&state.pool, assignment_id).await?;
    ensure_can_manage(state, auth, assignment.subject_id).await?;
    let res = sqlx::query(
        "UPDATE assignment_submissions SET is_active = FALSE
         WHERE id = $1 AND assignment_id = $2",
    )
    .bind(submission_id)
    .bind(assignment_id)
    .execute(&state.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Submission not found".into()));
    }
    Ok(())
}

pub async fn my_submissions(
    state: &AppState,
    auth: &AuthUser,
    q: &MySubmissionListQuery,
) -> AppResult<Paginated<Submission>> {
    ensure_role(auth, &[Role::Student])?;
    let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
    let p = Pagination::new(q.page, q.per_page);
    const FILTER: &str = "WHERE sub.student_id = $1 AND sub.is_active";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM assignment_submissions sub {FILTER}"
    ))
    .bind(student.id)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, Submission>(&format!(
        "{SUBMISSION_SELECT} {FILTER}
         ORDER BY sub.submitted_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(student.id)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}
