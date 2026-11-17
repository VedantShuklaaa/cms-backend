use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const AUDIENCES: &[&str] = &["ALL", "STUDENTS", "FACULTY", "ADMIN"];

#[derive(Debug, Serialize, FromRow)]
pub struct Notice {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub audience: String,
    pub created_by: Uuid,
    pub created_by_email: String,
    pub published_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoticeRequest {
    pub title: String,
    pub body: String,
    pub audience: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoticeRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub audience: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct NoticeListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub audience: Option<String>,
    pub include_inactive: Option<bool>,
    pub include_expired: Option<bool>,
}
