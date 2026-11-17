use crate::errors::{AppError, AppResult};

pub const MIN_PASSWORD_LEN: usize = 8;
/// Upper bound stops absurdly long inputs being fed to Argon2.
pub const MAX_PASSWORD_LEN: usize = 128;

pub fn normalize_email(raw: &str) -> String {
    raw.trim().to_lowercase()
}

pub fn validate_email(email: &str) -> AppResult<()> {
    let ok = email.len() <= 254
        && !email.chars().any(char::is_whitespace)
        && match email.split_once('@') {
            Some((local, domain)) => {
                !local.is_empty()
                    && !domain.contains('@')
                    && domain.contains('.')
                    && !domain.starts_with('.')
                    && !domain.ends_with('.')
            }
            None => false,
        };
    if ok {
        Ok(())
    } else {
        Err(AppError::Validation("Invalid email address".into()))
    }
}

pub fn validate_password(password: &str) -> AppResult<()> {
    let len = password.chars().count();
    if !(MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&len) {
        return Err(AppError::Validation(format!(
            "Password must be between {MIN_PASSWORD_LEN} and {MAX_PASSWORD_LEN} characters"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_normalization() {
        assert_eq!(normalize_email("  Admin@College.EDU "), "admin@college.edu");
    }

    #[test]
    fn email_validation() {
        assert!(validate_email("a@b.co").is_ok());
        for bad in [
            "", "a", "a@", "@b.co", "a@b", "a b@c.co", "a@@b.co", "a@.co", "a@b.",
        ] {
            assert!(validate_email(bad).is_err(), "{bad} should be invalid");
        }
    }

    #[test]
    fn password_length_bounds() {
        assert!(validate_password("1234567").is_err());
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password(&"a".repeat(MAX_PASSWORD_LEN)).is_ok());
        assert!(validate_password(&"a".repeat(MAX_PASSWORD_LEN + 1)).is_err());
    }
}
