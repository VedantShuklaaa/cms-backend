use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum Role {
    Admin,
    Faculty,
    Student,
}

/// Database row. Deliberately NOT Serialize so the hash can never be returned by accident.
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub password_changed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            role: u.role,
            is_active: u.is_active,
            created_at: u.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_serializes_uppercase() {
        assert_eq!(serde_json::to_string(&Role::Admin).unwrap(), "\"ADMIN\"");
        assert_eq!(
            serde_json::to_string(&Role::Student).unwrap(),
            "\"STUDENT\""
        );
    }

    #[test]
    fn role_rejects_unknown_value() {
        assert!(serde_json::from_str::<Role>("\"admin\"").is_err());
        assert!(serde_json::from_str::<Role>("\"ROOT\"").is_err());
    }
}
