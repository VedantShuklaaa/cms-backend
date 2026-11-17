use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Assignment {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub subject_name: String,
    pub subject_code: String,
    pub course_id: Uuid,
    pub faculty_id: Uuid,
    pub faculty_first_name: String,
    pub faculty_last_name: String,
    pub title: String,
    pub description: String,
    pub due_date: NaiveDate,
    pub max_marks: Option<i16>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssignmentRequest {
    pub subject_id: Uuid,
    /// Required when an admin creates on behalf of a faculty member.
    pub faculty_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub due_date: NaiveDate,
    pub max_marks: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssignmentRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub max_marks: Option<i16>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AssignmentListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub subject_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Submission {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub assignment_title: String,
    pub student_id: Uuid,
    pub student_user_id: Uuid,
    pub enrollment_no: String,
    pub student_first_name: String,
    pub student_last_name: String,
    pub content: String,
    pub submitted_at: DateTime<Utc>,
    pub marks_obtained: Option<i16>,
    pub graded_by: Option<Uuid>,
    pub graded_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct GradeSubmissionRequest {
    pub marks_obtained: Option<i16>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SubmissionListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub student_id: Option<Uuid>,
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MySubmissionListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
