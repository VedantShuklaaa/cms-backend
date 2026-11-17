use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middleware::rbac::ensure_role,
    models::{
        auth::AuthUser,
        report::{
            AttendanceReportQuery, AttendanceReportRow, FeeReportQuery, FeeReportRow,
            OverviewReport, ResultsReportQuery, ResultsReportRow,
        },
        user::Role,
    },
    services::lookups,
    state::AppState,
};

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

pub async fn overview(state: &AppState, auth: &AuthUser) -> AppResult<OverviewReport> {
    ensure_role(auth, &[Role::Admin])?;
    Ok(sqlx::query_as::<_, OverviewReport>(
        "SELECT
            (SELECT COUNT(*) FROM departments WHERE is_active)::bigint AS departments,
            (SELECT COUNT(*) FROM courses WHERE is_active)::bigint AS courses,
            (SELECT COUNT(*) FROM subjects WHERE is_active)::bigint AS subjects,
            (SELECT COUNT(*) FROM faculty f JOIN users u ON u.id = f.user_id
                WHERE u.is_active)::bigint AS faculty,
            (SELECT COUNT(*) FROM students s JOIN users u ON u.id = s.user_id
                WHERE u.is_active)::bigint AS students,
            (SELECT COUNT(*) FROM notices WHERE is_active)::bigint AS notices,
            (SELECT COUNT(*) FROM assignments WHERE is_active)::bigint AS active_assignments",
    )
    .fetch_one(&state.pool)
    .await?)
}

pub async fn attendance(
    state: &AppState,
    auth: &AuthUser,
    q: &AttendanceReportQuery,
) -> AppResult<Vec<AttendanceReportRow>> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    ensure_date_range(q.date_from, q.date_to)?;
    let is_admin = auth.role == Role::Admin;
    let faculty_id = if is_admin {
        Uuid::nil()
    } else {
        lookups::faculty_id_for_user(&state.pool, auth.id).await?
    };

    Ok(sqlx::query_as::<_, AttendanceReportRow>(
        "SELECT s.id AS subject_id, s.code AS subject_code, s.name AS subject_name,
                c.name AS course_name,
                COUNT(*)::bigint AS total,
                (COUNT(*) FILTER (WHERE a.status = 'PRESENT'))::bigint AS present,
                (COUNT(*) FILTER (WHERE a.status = 'ABSENT'))::bigint AS absent,
                (COUNT(*) FILTER (WHERE a.status = 'LATE'))::bigint AS late,
                (COUNT(*) FILTER (WHERE a.status = 'EXCUSED'))::bigint AS excused,
                COALESCE(ROUND(
                    100.0 * (COUNT(*) FILTER (WHERE a.status IN ('PRESENT', 'LATE')))
                        ::numeric / NULLIF(COUNT(*), 0), 2), 0)::float8 AS attendance_percent
         FROM attendance a
         JOIN subjects s ON s.id = a.subject_id
         JOIN courses c ON c.id = s.course_id
         WHERE a.is_active
           AND ($1::uuid IS NULL OR a.subject_id = $1)
           AND ($2::uuid IS NULL OR s.course_id = $2)
           AND ($3::uuid IS NULL OR c.department_id = $3)
           AND ($4::date IS NULL OR a.attendance_date >= $4)
           AND ($5::date IS NULL OR a.attendance_date <= $5)
           AND ($6::bool OR EXISTS (
               SELECT 1 FROM faculty_subjects fs
               WHERE fs.faculty_id = $7 AND fs.subject_id = a.subject_id
           ))
         GROUP BY s.id, s.code, s.name, c.name
         ORDER BY s.name",
    )
    .bind(q.subject_id)
    .bind(q.course_id)
    .bind(q.department_id)
    .bind(q.date_from)
    .bind(q.date_to)
    .bind(is_admin)
    .bind(faculty_id)
    .fetch_all(&state.pool)
    .await?)
}

pub async fn results(
    state: &AppState,
    auth: &AuthUser,
    q: &ResultsReportQuery,
) -> AppResult<Vec<ResultsReportRow>> {
    ensure_role(auth, &[Role::Admin, Role::Faculty])?;
    let is_admin = auth.role == Role::Admin;
    let faculty_id = if is_admin {
        Uuid::nil()
    } else {
        lookups::faculty_id_for_user(&state.pool, auth.id).await?
    };

    Ok(sqlx::query_as::<_, ResultsReportRow>(
        "SELECT e.id AS exam_id, e.name AS exam_name, s.id AS subject_id,
                s.code AS subject_code, s.name AS subject_name,
                COUNT(*)::bigint AS students_graded,
                COALESCE(AVG(r.marks_obtained::float8), 0)::float8 AS average_marks,
                COALESCE(MAX(r.marks_obtained), 0)::smallint AS highest_marks,
                COALESCE(MIN(r.marks_obtained), 0)::smallint AS lowest_marks,
                e.max_marks,
                COALESCE(ROUND(
                    100.0 * (COUNT(*) FILTER (WHERE r.marks_obtained >= 0.4 * e.max_marks))
                        ::numeric / NULLIF(COUNT(*), 0), 2), 0)::float8 AS pass_rate
         FROM results r
         JOIN exams e ON e.id = r.exam_id
         JOIN subjects s ON s.id = e.subject_id
         WHERE r.is_active AND e.is_active
           AND ($1::uuid IS NULL OR r.exam_id = $1)
           AND ($2::uuid IS NULL OR e.subject_id = $2)
           AND ($3::uuid IS NULL OR s.course_id = $3)
           AND ($4::bool OR EXISTS (
               SELECT 1 FROM faculty_subjects fs
               WHERE fs.faculty_id = $5 AND fs.subject_id = e.subject_id
           ))
         GROUP BY e.id, e.name, s.id, s.code, s.name, e.max_marks
         ORDER BY e.exam_date DESC, e.name",
    )
    .bind(q.exam_id)
    .bind(q.subject_id)
    .bind(q.course_id)
    .bind(is_admin)
    .bind(faculty_id)
    .fetch_all(&state.pool)
    .await?)
}

pub async fn fees(
    state: &AppState,
    auth: &AuthUser,
    q: &FeeReportQuery,
) -> AppResult<Vec<FeeReportRow>> {
    ensure_role(auth, &[Role::Admin])?;
    Ok(sqlx::query_as::<_, FeeReportRow>(
        "SELECT f.id AS fee_structure_id, f.name, c.name AS course_name,
                f.academic_year, f.amount,
                COALESCE(SUM(p.amount) FILTER (WHERE p.is_active), 0)::bigint AS collected,
                GREATEST(f.amount - COALESCE(SUM(p.amount) FILTER (WHERE p.is_active), 0), 0)
                    ::bigint AS outstanding,
                (COUNT(p.id) FILTER (WHERE p.is_active))::bigint AS payment_count
         FROM fee_structures f
         JOIN courses c ON c.id = f.course_id
         LEFT JOIN fee_payments p ON p.fee_structure_id = f.id
         WHERE f.is_active
           AND ($1::uuid IS NULL OR f.course_id = $1)
           AND ($2::smallint IS NULL OR f.academic_year = $2)
         GROUP BY f.id, f.name, c.name, f.academic_year, f.amount
         ORDER BY f.academic_year DESC, f.name",
    )
    .bind(q.course_id)
    .bind(q.academic_year)
    .fetch_all(&state.pool)
    .await?)
}
