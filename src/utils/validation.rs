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

/// Trims and checks 1..=max characters.
pub fn clean_text(field: &str, raw: &str, max: usize) -> AppResult<String> {
    let t = raw.trim();
    if t.is_empty() || t.chars().count() > max {
        return Err(AppError::Validation(format!(
            "{field} must be between 1 and {max} characters"
        )));
    }
    Ok(t.to_string())
}

/// None stays None. A provided value must be non-empty and within max.
pub fn clean_optional_text(
    field: &str,
    raw: Option<&str>,
    max: usize,
) -> AppResult<Option<String>> {
    raw.map(|v| clean_text(field, v, max)).transpose()
}

/// Uppercased identifier: 2 to 20 chars of A-Z, 0-9 and '-'.
pub fn clean_code(field: &str, raw: &str) -> AppResult<String> {
    let code = raw.trim().to_uppercase();
    let ok = (2..=20).contains(&code.len())
        && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if ok {
        Ok(code)
    } else {
        Err(AppError::Validation(format!(
            "{field} must be 2 to 20 characters: letters, digits or '-'"
        )))
    }
}

#[cfg(test)]
mod text_tests {
    use super::*;

    #[test]
    fn text_trims_and_bounds() {
        assert_eq!(clean_text("n", "  CS  ", 10).unwrap(), "CS");
        assert!(clean_text("n", "   ", 10).is_err());
        assert!(clean_text("n", "12345678901", 10).is_err());
    }

    #[test]
    fn optional_text() {
        assert_eq!(clean_optional_text("d", None, 5).unwrap(), None);
        assert!(clean_optional_text("d", Some(""), 5).is_err());
        assert_eq!(
            clean_optional_text("d", Some(" ab "), 5).unwrap().unwrap(),
            "ab"
        );
    }

    #[test]
    fn code_rules() {
        assert_eq!(clean_code("c", " cs-101 ").unwrap(), "CS-101");
        for bad in ["", "A", "has space", "a_b", "a/b", &"X".repeat(21)] {
            assert!(clean_code("c", bad).is_err(), "{bad} should be invalid");
        }
    }
}
