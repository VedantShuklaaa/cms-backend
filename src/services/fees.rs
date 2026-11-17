use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult, conflict_on_unique},
    middleware::rbac::ensure_role,
    models::{
        auth::AuthUser,
        fee::{
            CreateFeePaymentRequest, CreateFeeStructureRequest, FeePayment, FeePaymentListQuery,
            FeeStatement, FeeStatementListQuery, FeeStructure, FeeStructureListQuery,
            MyPaymentListQuery, PAYMENT_METHODS, UpdateFeeStructureRequest,
        },
        user::Role,
    },
    services::{lookups, students as student_svc},
    state::AppState,
    utils::{
        query::{Paginated, Pagination, like_pattern},
        validation::{clean_choice, clean_optional_text, clean_text},
    },
};

const FEE_SELECT: &str = "SELECT f.id, f.course_id, c.name AS course_name, f.academic_year,
    f.name, f.amount, f.due_date, f.is_active, f.created_at, f.updated_at
    FROM fee_structures f JOIN courses c ON c.id = f.course_id";
const FEE_DUPLICATE: &str = "A fee with this name already exists for this course and year";

const PAYMENT_SELECT: &str = "SELECT p.id, p.fee_structure_id, p.student_id,
    st.enrollment_no, st.first_name AS student_first_name,
    st.last_name AS student_last_name, p.amount, p.paid_on, p.method, p.reference,
    p.recorded_by, p.is_active, p.created_at, p.updated_at
    FROM fee_payments p JOIN students st ON st.id = p.student_id";

fn validate_year(year: i16) -> AppResult<i16> {
    if (1950..=2200).contains(&year) {
        Ok(year)
    } else {
        Err(AppError::Validation("academic_year is out of range".into()))
    }
}

fn validate_amount(amount: i64) -> AppResult<i64> {
    if amount > 0 {
        Ok(amount)
    } else {
        Err(AppError::Validation("amount must be greater than 0".into()))
    }
}

async fn fetch_fee(pool: &PgPool, id: Uuid) -> AppResult<FeeStructure> {
    sqlx::query_as::<_, FeeStructure>(&format!("{FEE_SELECT} WHERE f.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Fee structure not found".into()))
}

async fn fetch_payment(pool: &PgPool, id: Uuid) -> AppResult<FeePayment> {
    sqlx::query_as::<_, FeePayment>(&format!("{PAYMENT_SELECT} WHERE p.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Payment not found".into()))
}

pub async fn list(
    state: &AppState,
    auth: &AuthUser,
    q: &FeeStructureListQuery,
) -> AppResult<Paginated<FeeStructure>> {
    let p = Pagination::new(q.page, q.per_page);
    let is_admin = auth.role == Role::Admin;
    let include_inactive = q.include_inactive.unwrap_or(false) && is_admin;
    let search = like_pattern(q.search.as_deref());

    let course_id = if auth.role == Role::Student {
        let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
        if let Some(requested) = q.course_id
            && requested != student.course_id
        {
            return Err(AppError::Forbidden(
                "You can only view fees for your own course".into(),
            ));
        }
        Some(student.course_id)
    } else {
        q.course_id
    };

    const FILTER: &str = "WHERE ($1::bool OR f.is_active)
        AND ($2::uuid IS NULL OR f.course_id = $2)
        AND ($3::smallint IS NULL OR f.academic_year = $3)
        AND ($4::text IS NULL OR f.name ILIKE $4 OR c.name ILIKE $4)";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM fee_structures f JOIN courses c ON c.id = f.course_id {FILTER}"
    ))
    .bind(include_inactive)
    .bind(course_id)
    .bind(q.academic_year)
    .bind(&search)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, FeeStructure>(&format!(
        "{FEE_SELECT} {FILTER} ORDER BY f.academic_year DESC, f.name LIMIT $5 OFFSET $6"
    ))
    .bind(include_inactive)
    .bind(course_id)
    .bind(q.academic_year)
    .bind(&search)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn get(state: &AppState, auth: &AuthUser, id: Uuid) -> AppResult<FeeStructure> {
    let fee = fetch_fee(&state.pool, id).await?;
    if auth.role != Role::Admin && !fee.is_active {
        return Err(AppError::NotFound("Fee structure not found".into()));
    }
    if auth.role == Role::Student {
        let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
        if student.course_id != fee.course_id {
            return Err(AppError::Forbidden(
                "You can only view fees for your own course".into(),
            ));
        }
    }
    Ok(fee)
}

pub async fn create(state: &AppState, req: CreateFeeStructureRequest) -> AppResult<FeeStructure> {
    let name = clean_text("name", &req.name, 150)?;
    let year = validate_year(req.academic_year)?;
    let amount = validate_amount(req.amount)?;
    lookups::active_course_duration(&state.pool, req.course_id).await?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO fee_structures (course_id, academic_year, name, amount, due_date)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(req.course_id)
    .bind(year)
    .bind(name)
    .bind(amount)
    .bind(req.due_date)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, FEE_DUPLICATE))?;

    fetch_fee(&state.pool, id).await
}

pub async fn update(
    state: &AppState,
    id: Uuid,
    req: UpdateFeeStructureRequest,
) -> AppResult<FeeStructure> {
    let name = req
        .name
        .as_deref()
        .map(|v| clean_text("name", v, 150))
        .transpose()?;
    let year = req.academic_year.map(validate_year).transpose()?;
    let amount = req.amount.map(validate_amount).transpose()?;
    if let Some(course_id) = req.course_id {
        lookups::active_course_duration(&state.pool, course_id).await?;
    }

    if let Some(new_amount) = amount {
        let over: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM fee_structures f
                WHERE f.id = $1 AND EXISTS (
                    SELECT 1 FROM fee_payments p
                    WHERE p.fee_structure_id = f.id AND p.is_active
                    GROUP BY p.student_id
                    HAVING SUM(p.amount) > $2
                )
            )",
        )
        .bind(id)
        .bind(new_amount)
        .fetch_one(&state.pool)
        .await?;
        if over {
            return Err(AppError::Validation(
                "amount is lower than a student's payments already recorded".into(),
            ));
        }
    }

    let updated: Option<Uuid> = sqlx::query_scalar(
        "UPDATE fee_structures SET
            course_id = COALESCE($2, course_id),
            academic_year = COALESCE($3, academic_year),
            name = COALESCE($4, name),
            amount = COALESCE($5, amount),
            due_date = COALESCE($6, due_date),
            is_active = COALESCE($7, is_active)
         WHERE id = $1 RETURNING id",
    )
    .bind(id)
    .bind(req.course_id)
    .bind(year)
    .bind(name)
    .bind(amount)
    .bind(req.due_date)
    .bind(req.is_active)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| conflict_on_unique(e, FEE_DUPLICATE))?;

    updated.ok_or_else(|| AppError::NotFound("Fee structure not found".into()))?;
    fetch_fee(&state.pool, id).await
}

/// Soft delete. Refuses while active payments exist.
pub async fn deactivate(state: &AppState, id: Uuid) -> AppResult<()> {
    fetch_fee(&state.pool, id).await?;
    let has_payments: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM fee_payments WHERE fee_structure_id = $1 AND is_active)",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    if has_payments {
        return Err(AppError::Conflict(
            "Fee structure has active payments; remove or deactivate them first".into(),
        ));
    }
    sqlx::query("UPDATE fee_structures SET is_active = FALSE WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(())
}

pub async fn list_payments(
    state: &AppState,
    auth: &AuthUser,
    fee_structure_id: Uuid,
    q: &FeePaymentListQuery,
) -> AppResult<Paginated<FeePayment>> {
    ensure_role(auth, &[Role::Admin])?;
    fetch_fee(&state.pool, fee_structure_id).await?;
    let p = Pagination::new(q.page, q.per_page);
    let include_inactive = q.include_inactive.unwrap_or(false);
    const FILTER: &str = "WHERE p.fee_structure_id = $1
        AND ($2::bool OR p.is_active)
        AND ($3::uuid IS NULL OR p.student_id = $3)";

    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM fee_payments p {FILTER}"))
        .bind(fee_structure_id)
        .bind(include_inactive)
        .bind(q.student_id)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, FeePayment>(&format!(
        "{PAYMENT_SELECT} {FILTER} ORDER BY p.paid_on DESC, p.created_at DESC
         LIMIT $4 OFFSET $5"
    ))
    .bind(fee_structure_id)
    .bind(include_inactive)
    .bind(q.student_id)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn create_payment(
    state: &AppState,
    auth: &AuthUser,
    fee_structure_id: Uuid,
    req: CreateFeePaymentRequest,
) -> AppResult<FeePayment> {
    ensure_role(auth, &[Role::Admin])?;
    let fee = fetch_fee(&state.pool, fee_structure_id).await?;
    if !fee.is_active {
        return Err(AppError::Validation("Fee structure is inactive".into()));
    }
    let student_course = lookups::active_student_course(&state.pool, req.student_id).await?;
    if student_course != fee.course_id {
        return Err(AppError::Validation(
            "Student is not enrolled in the fee's course".into(),
        ));
    }
    let amount = validate_amount(req.amount)?;
    if req.paid_on > Utc::now().date_naive() {
        return Err(AppError::Validation(
            "paid_on cannot be in the future".into(),
        ));
    }
    let method = clean_choice("method", &req.method, PAYMENT_METHODS)?;
    let reference = clean_optional_text("reference", req.reference.as_deref(), 100)?;

    let paid_so_far: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0)::bigint FROM fee_payments
         WHERE fee_structure_id = $1 AND student_id = $2 AND is_active",
    )
    .bind(fee_structure_id)
    .bind(req.student_id)
    .fetch_one(&state.pool)
    .await?;
    if paid_so_far + amount > fee.amount {
        return Err(AppError::Validation(
            "Payment exceeds the student's outstanding balance".into(),
        ));
    }

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO fee_payments (fee_structure_id, student_id, amount, paid_on, method,
                                   reference, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(fee_structure_id)
    .bind(req.student_id)
    .bind(amount)
    .bind(req.paid_on)
    .bind(method)
    .bind(reference)
    .bind(auth.id)
    .fetch_one(&state.pool)
    .await?;

    fetch_payment(&state.pool, id).await
}

/// Soft delete.
pub async fn deactivate_payment(
    state: &AppState,
    auth: &AuthUser,
    fee_structure_id: Uuid,
    payment_id: Uuid,
) -> AppResult<()> {
    ensure_role(auth, &[Role::Admin])?;
    let res = sqlx::query(
        "UPDATE fee_payments SET is_active = FALSE
         WHERE id = $1 AND fee_structure_id = $2",
    )
    .bind(payment_id)
    .bind(fee_structure_id)
    .execute(&state.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Payment not found".into()));
    }
    Ok(())
}

pub async fn my_statements(
    state: &AppState,
    auth: &AuthUser,
    q: &FeeStatementListQuery,
) -> AppResult<Paginated<FeeStatement>> {
    ensure_role(auth, &[Role::Student])?;
    let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
    let p = Pagination::new(q.page, q.per_page);
    const FILTER: &str = "WHERE f.course_id = $1 AND f.is_active
        AND ($2::smallint IS NULL OR f.academic_year = $2)";

    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM fee_structures f {FILTER}"))
        .bind(student.course_id)
        .bind(q.academic_year)
        .fetch_one(&state.pool)
        .await?;

    let items = sqlx::query_as::<_, FeeStatement>(&format!(
        "SELECT f.id AS fee_structure_id, f.course_id, c.name AS course_name,
                f.academic_year, f.name, f.amount, f.due_date,
                COALESCE(SUM(p.amount) FILTER (WHERE p.is_active), 0)::bigint AS paid_amount,
                (f.amount - COALESCE(SUM(p.amount) FILTER (WHERE p.is_active), 0))::bigint
                    AS balance
         FROM fee_structures f
         JOIN courses c ON c.id = f.course_id
         LEFT JOIN fee_payments p
             ON p.fee_structure_id = f.id AND p.student_id = $3
         {FILTER}
         GROUP BY f.id, f.course_id, c.name, f.academic_year, f.name, f.amount, f.due_date
         ORDER BY f.academic_year DESC, f.name LIMIT $4 OFFSET $5"
    ))
    .bind(student.course_id)
    .bind(q.academic_year)
    .bind(student.id)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}

pub async fn my_payments(
    state: &AppState,
    auth: &AuthUser,
    q: &MyPaymentListQuery,
) -> AppResult<Paginated<FeePayment>> {
    ensure_role(auth, &[Role::Student])?;
    let student = student_svc::fetch_by_user(&state.pool, auth.id).await?;
    let p = Pagination::new(q.page, q.per_page);
    const FILTER: &str = "WHERE p.student_id = $1 AND p.is_active
        AND ($2::smallint IS NULL OR f.academic_year = $2)";

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM fee_payments p
         JOIN fee_structures f ON f.id = p.fee_structure_id {FILTER}"
    ))
    .bind(student.id)
    .bind(q.academic_year)
    .fetch_one(&state.pool)
    .await?;

    let items = sqlx::query_as::<_, FeePayment>(&format!(
        "{PAYMENT_SELECT}
         JOIN fee_structures f ON f.id = p.fee_structure_id
         {FILTER} ORDER BY p.paid_on DESC, p.created_at DESC LIMIT $3 OFFSET $4"
    ))
    .bind(student.id)
    .bind(q.academic_year)
    .bind(p.limit())
    .bind(p.offset())
    .fetch_all(&state.pool)
    .await?;

    Ok(Paginated::new(items, total, p))
}
