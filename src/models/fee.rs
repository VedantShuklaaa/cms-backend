use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const PAYMENT_METHODS: &[&str] = &["CASH", "CARD", "UPI", "BANK_TRANSFER"];

#[derive(Debug, Serialize, FromRow)]
pub struct FeeStructure {
    pub id: Uuid,
    pub course_id: Uuid,
    pub course_name: String,
    pub academic_year: i16,
    pub name: String,
    pub amount: i64,
    pub due_date: NaiveDate,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFeeStructureRequest {
    pub course_id: Uuid,
    pub academic_year: i16,
    pub name: String,
    pub amount: i64,
    pub due_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFeeStructureRequest {
    pub course_id: Option<Uuid>,
    pub academic_year: Option<i16>,
    pub name: Option<String>,
    pub amount: Option<i64>,
    pub due_date: Option<NaiveDate>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FeeStructureListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub course_id: Option<Uuid>,
    pub academic_year: Option<i16>,
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FeePayment {
    pub id: Uuid,
    pub fee_structure_id: Uuid,
    pub student_id: Uuid,
    pub enrollment_no: String,
    pub student_first_name: String,
    pub student_last_name: String,
    pub amount: i64,
    pub paid_on: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
    pub recorded_by: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFeePaymentRequest {
    pub student_id: Uuid,
    pub amount: i64,
    pub paid_on: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeePaymentListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub student_id: Option<Uuid>,
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FeeStatement {
    pub fee_structure_id: Uuid,
    pub course_id: Uuid,
    pub course_name: String,
    pub academic_year: i16,
    pub name: String,
    pub amount: i64,
    pub due_date: NaiveDate,
    pub paid_amount: i64,
    pub balance: i64,
}

#[derive(Debug, Deserialize)]
pub struct FeeStatementListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub academic_year: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct MyPaymentListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub academic_year: Option<i16>,
}
