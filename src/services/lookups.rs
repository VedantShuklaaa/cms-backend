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
