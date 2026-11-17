use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Faculty {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub department_id: Uuid,
    pub department_name: String,
    pub employee_id: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub designation: String,
    pub joined_on: Option<NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFacultyRequest {
    pub email: String,
    pub password: String,
    pub department_id: Uuid,
    pub employee_id: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub designation: String,
    pub joined_on: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFacultyRequest {
    pub department_id: Option<Uuid>,
    pub employee_id: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub designation: Option<String>,
    pub joined_on: Option<NaiveDate>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FacultyListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub department_id: Option<Uuid>,
    pub include_inactive: Option<bool>,
}
