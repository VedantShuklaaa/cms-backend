use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const EXAM_TYPES: &[&str] = &["QUIZ", "MIDTERM", "FINAL", "PRACTICAL"];

#[derive(Debug, Serialize, FromRow)]
pub struct Exam {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub subject_name: String,
    pub subject_code: String,
    pub course_id: Uuid,
    pub course_name: String,
    pub name: String,
    pub exam_type: String,
    pub exam_date: NaiveDate,
    pub max_marks: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateExamRequest {
    pub subject_id: Uuid,
    pub name: String,
    pub exam_type: String,
    pub exam_date: NaiveDate,
    pub max_marks: i16,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExamRequest {
    pub subject_id: Option<Uuid>,
    pub name: Option<String>,
    pub exam_type: Option<String>,
    pub exam_date: Option<NaiveDate>,
    pub max_marks: Option<i16>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ExamListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub subject_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub exam_type: Option<String>,
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Result {
    pub id: Uuid,
    pub exam_id: Uuid,
    pub exam_name: String,
    pub exam_type: String,
    pub subject_id: Uuid,
    pub subject_name: String,
    pub student_id: Uuid,
    pub student_user_id: Uuid,
    pub enrollment_no: String,
    pub student_first_name: String,
    pub student_last_name: String,
    pub marks_obtained: i16,
    pub max_marks: i16,
    pub grade: Option<String>,
    pub recorded_by: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateResultRequest {
    pub exam_id: Uuid,
    pub student_id: Uuid,
    pub marks_obtained: i16,
    pub grade: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateResultRequest {
    pub marks_obtained: Option<i16>,
    pub grade: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ResultListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub exam_id: Option<Uuid>,
    pub subject_id: Option<Uuid>,
    pub student_id: Option<Uuid>,
    pub include_inactive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MyResultQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub subject_id: Option<Uuid>,
    pub exam_id: Option<Uuid>,
}
