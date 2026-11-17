use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const ATTENDANCE_STATUSES: &[&str] = &["PRESENT", "ABSENT", "LATE", "EXCUSED"];

#[derive(Debug, Serialize, FromRow)]
pub struct Attendance {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub subject_name: String,
    pub subject_code: String,
    pub student_id: Uuid,
    pub enrollment_no: String,
    pub student_first_name: String,
    pub student_last_name: String,
    pub marked_by: Option<Uuid>,
    pub attendance_date: NaiveDate,
    pub status: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAttendanceRequest {
    pub subject_id: Uuid,
    pub student_id: Uuid,
    pub attendance_date: NaiveDate,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAttendanceRequest {
    pub attendance_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AttendanceListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub subject_id: Option<Uuid>,
    pub student_id: Option<Uuid>,
    pub status: Option<String>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub include_inactive: Option<bool>,
}

/// Student self-service list: same filters minus student_id.
#[derive(Debug, Deserialize)]
pub struct MyAttendanceQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub subject_id: Option<Uuid>,
    pub status: Option<String>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
}
