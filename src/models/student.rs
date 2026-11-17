use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Student {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub course_id: Uuid,
    pub course_name: String,
    pub department_id: Uuid,
    pub department_name: String,
    pub enrollment_no: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub current_semester: i16,
    pub admission_year: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStudentRequest {
    pub email: String,
    pub password: String,
    pub course_id: Uuid,
    pub enrollment_no: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub current_semester: i16,
    pub admission_year: i16,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStudentRequest {
    pub course_id: Option<Uuid>,
    pub enrollment_no: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub current_semester: Option<i16>,
    pub admission_year: Option<i16>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct StudentListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub department_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub semester: Option<i16>,
    pub include_inactive: Option<bool>,
}
