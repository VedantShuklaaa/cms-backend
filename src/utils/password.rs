use std::sync::OnceLock;

use argon2::{
    Argon2,
    password_hash::{
        Error as HashError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
};

use crate::errors::{AppError, AppResult};

/// Argon2id with the crate's default parameters.
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("password hashing failed: {e}")))
}

/// Ok(false) = wrong password. Err = stored hash is malformed or hashing failed.
pub fn verify_password(password: &str, stored_hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(stored_hash)
        .map_err(|e| AppError::Internal(format!("stored password hash is invalid: {e}")))?;
    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(HashError::Password) => Ok(false),
        Err(e) => Err(AppError::Internal(format!(
            "password verification failed: {e}"
        ))),
    }
}

/// Hashing is CPU heavy, so keep it off the async worker threads.
pub async fn hash(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|e| AppError::Internal(format!("hash task failed: {e}")))?
}

pub async fn verify(password: String, stored_hash: String) -> AppResult<bool> {
    tokio::task::spawn_blocking(move || verify_password(&password, &stored_hash))
        .await
        .map_err(|e| AppError::Internal(format!("verify task failed: {e}")))?
}

/// Valid hash used to equalise timing when the login email doesn't exist.
pub fn dummy_hash() -> &'static str {
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY
        .get_or_init(|| hash_password("dummy-password-for-timing").unwrap_or_default())
        .as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_succeeds() {
        let hash = hash_password("correct horse battery").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password("correct horse battery", &hash).unwrap());
    }

    #[test]
    fn wrong_password_fails() {
        let hash = hash_password("correct horse battery").unwrap();
        assert!(!verify_password("wrong password", &hash).unwrap());
    }

    #[test]
    fn same_password_gets_different_salts() {
        let a = hash_password("same-password").unwrap();
        let b = hash_password("same-password").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn malformed_hash_is_an_error() {
        assert!(verify_password("x", "not-a-hash").is_err());
    }

    #[test]
    fn dummy_hash_is_valid() {
        assert!(!verify_password("nope", dummy_hash()).unwrap());
    }
}
