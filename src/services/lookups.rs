use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

pub async fn ensure_active_department(pool: &PgPool, id: Uuid) -> AppResult<()> {
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

/// Returns the course's duration in semesters.
pub async fn active_course_duration(pool: &PgPool, id: Uuid) -> AppResult<i16> {
    sqlx::query_scalar::<_, i16>(
        "SELECT duration_semesters FROM courses WHERE id = $1 AND is_active",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Validation("Course does not exist or is inactive".into()))
}

/// Validates that a subject is active and returns its course_id.
pub async fn active_subject_course(pool: &PgPool, id: Uuid) -> AppResult<Uuid> {
    sqlx::query_scalar("SELECT course_id FROM subjects WHERE id = $1 AND is_active")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation("Subject does not exist or is inactive".into()))
}

/// Validates that a student account is active and returns the student's course_id.
pub async fn active_student_course(pool: &PgPool, id: Uuid) -> AppResult<Uuid> {
    sqlx::query_scalar(
        "SELECT s.course_id FROM students s JOIN users u ON u.id = s.user_id
         WHERE s.id = $1 AND u.is_active",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Validation("Student does not exist or is inactive".into()))
}

/// Returns (course_id, user_id) for a student row, regardless of account state.
pub async fn student_course_and_user(pool: &PgPool, id: Uuid) -> AppResult<(Uuid, Uuid)> {
    sqlx::query_as("SELECT course_id, user_id FROM students WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Student not found".into()))
}

pub async fn ensure_active_faculty(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let ok: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM faculty f JOIN users u ON u.id = f.user_id
                       WHERE f.id = $1 AND u.is_active)",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    if ok {
        Ok(())
    } else {
        Err(AppError::Validation(
            "Faculty member does not exist or is inactive".into(),
        ))
    }
}

/// Faculty row id for an authenticated faculty user.
pub async fn faculty_id_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<Uuid> {
    sqlx::query_scalar(
        "SELECT f.id FROM faculty f JOIN users u ON u.id = f.user_id
         WHERE f.user_id = $1 AND u.is_active",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Validation("Faculty profile does not exist or is inactive".into()))
}

pub async fn is_faculty_assigned(
    pool: &PgPool,
    faculty_id: Uuid,
    subject_id: Uuid,
) -> AppResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM faculty_subjects
                       WHERE faculty_id = $1 AND subject_id = $2)",
    )
    .bind(faculty_id)
    .bind(subject_id)
    .fetch_one(pool)
    .await?)
}

/// Validates that a faculty member teaches the subject.
pub async fn ensure_faculty_assigned(
    pool: &PgPool,
    faculty_id: Uuid,
    subject_id: Uuid,
) -> AppResult<()> {
    if is_faculty_assigned(pool, faculty_id, subject_id).await? {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "You are not assigned to this subject".into(),
        ))
    }
}

/// Validates that an exam is active and returns (subject_id, max_marks).
pub async fn active_exam_subject(pool: &PgPool, id: Uuid) -> AppResult<(Uuid, i16)> {
    sqlx::query_as("SELECT subject_id, max_marks FROM exams WHERE id = $1 AND is_active")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Validation("Exam does not exist or is inactive".into()))
}
