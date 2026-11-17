use sqlx::{PgExecutor, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    errors::{AppResult, conflict_on_unique},
    models::user::Role,
    utils::{password, validation},
};

/// Validated email + hashed password, prepared BEFORE a transaction opens
/// so Argon2 doesn't hold a DB connection while it runs.
pub struct NewAccount {
    email: String,
    password_hash: String,
}

impl NewAccount {
    pub async fn prepare(email: &str, pw: &str) -> AppResult<Self> {
        let email = validation::normalize_email(email);
        validation::validate_email(&email)?;
        validation::validate_password(pw)?;
        let password_hash = password::hash(pw.to_string()).await?;
        Ok(Self {
            email,
            password_hash,
        })
    }
}

pub async fn insert_account(
    tx: &mut Transaction<'_, Postgres>,
    acct: &NewAccount,
    role: Role,
) -> AppResult<Uuid> {
    sqlx::query_scalar(
        "INSERT INTO users (email, password_hash, role) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&acct.email)
    .bind(&acct.password_hash)
    .bind(role)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| conflict_on_unique(e, "An account with this email already exists"))
}

pub async fn set_active<'e, E: PgExecutor<'e>>(
    ex: E,
    user_id: Uuid,
    active: bool,
) -> AppResult<()> {
    sqlx::query("UPDATE users SET is_active = $2 WHERE id = $1")
        .bind(user_id)
        .bind(active)
        .execute(ex)
        .await?;
    Ok(())
}
