use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Course {
    pub id: Uuid,
    pub department_id: Uuid,
    pub department_name: String,
    pub name: String,
    pub code: String,
    pub duration_semesters: i16,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCourseRequest {
    pub department_id: Uuid,
    pub name: String,
    pub code: String,
    pub duration_semesters: i16,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCourseRequest {
    pub department_id: Option<Uuid>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub duration_semesters: Option<i16>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CourseListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub department_id: Option<Uuid>,
    pub include_inactive: Option<bool>,
}
