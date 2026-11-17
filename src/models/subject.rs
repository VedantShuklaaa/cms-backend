use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Subject {
    pub id: Uuid,
    pub course_id: Uuid,
    pub course_name: String,
    pub department_id: Uuid,
    pub department_name: String,
    pub name: String,
    pub code: String,
    pub semester: i16,
    pub credits: i16,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubjectRequest {
    pub course_id: Uuid,
    pub name: String,
    pub code: String,
    pub semester: i16,
    pub credits: i16,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubjectRequest {
    pub course_id: Option<Uuid>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub semester: Option<i16>,
    pub credits: Option<i16>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SubjectListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub course_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub semester: Option<i16>,
    /// Restrict to subjects assigned to this faculty member.
    pub faculty_id: Option<Uuid>,
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MySubjectQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignFacultyRequest {
    pub faculty_id: Uuid,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SubjectFaculty {
    pub subject_id: Uuid,
    pub faculty_id: Uuid,
    pub employee_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department_id: Uuid,
    pub department_name: String,
    pub assigned_at: DateTime<Utc>,
}
