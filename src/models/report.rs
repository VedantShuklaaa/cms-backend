use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct OverviewReport {
    pub departments: i64,
    pub courses: i64,
    pub subjects: i64,
    pub faculty: i64,
    pub students: i64,
    pub notices: i64,
    pub active_assignments: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AttendanceReportRow {
    pub subject_id: Uuid,
    pub subject_code: String,
    pub subject_name: String,
    pub course_name: String,
    pub total: i64,
    pub present: i64,
    pub absent: i64,
    pub late: i64,
    pub excused: i64,
    pub attendance_percent: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ResultsReportRow {
    pub exam_id: Uuid,
    pub exam_name: String,
    pub subject_id: Uuid,
    pub subject_code: String,
    pub subject_name: String,
    pub students_graded: i64,
    pub average_marks: f64,
    pub highest_marks: i16,
    pub lowest_marks: i16,
    pub max_marks: i16,
    pub pass_rate: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct FeeReportRow {
    pub fee_structure_id: Uuid,
    pub name: String,
    pub course_name: String,
    pub academic_year: i16,
    pub amount: i64,
    pub collected: i64,
    pub outstanding: i64,
    pub payment_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct AttendanceReportQuery {
    pub subject_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ResultsReportQuery {
    pub exam_id: Option<Uuid>,
    pub subject_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct FeeReportQuery {
    pub course_id: Option<Uuid>,
    pub academic_year: Option<i16>,
}
