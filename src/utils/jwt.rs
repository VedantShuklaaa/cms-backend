use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub jti: Uuid,
    pub iat: i64,
    pub exp: i64,
}

pub struct IssuedToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub fn issue_token(user_id: Uuid, secret: &str, ttl_minutes: i64) -> AppResult<IssuedToken> {
    let now = Utc::now();
    let expires_at = now + Duration::minutes(ttl_minutes);
    let claims = Claims {
        sub: user_id,
        jti: Uuid::new_v4(),
        iat: now.timestamp(),
        exp: expires_at.timestamp(),
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("jwt encoding failed: {e}")))?;
    Ok(IssuedToken { token, expires_at })
}

pub fn decode_token(token: &str, secret: &str) -> AppResult<Claims> {
    let mut validation = Validation::new(Algorithm::HS256); // only HS256 is accepted
    validation.leeway = 5;
    validation.set_required_spec_claims(&["exp", "sub"]);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-test-secret-test-secret-123";

    #[test]
    fn roundtrip() {
        let id = Uuid::new_v4();
        let issued = issue_token(id, SECRET, 60).unwrap();
        let claims = decode_token(&issued.token, SECRET).unwrap();
        assert_eq!(claims.sub, id);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let issued = issue_token(Uuid::new_v4(), SECRET, 60).unwrap();
        assert!(decode_token(&issued.token, "another-secret-another-secret-1234").is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let issued = issue_token(Uuid::new_v4(), SECRET, -10).unwrap();
        assert!(decode_token(&issued.token, SECRET).is_err());
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(decode_token("not.a.jwt", SECRET).is_err());
    }

    #[test]
    fn each_token_has_unique_jti() {
        let id = Uuid::new_v4();
        let a = decode_token(&issue_token(id, SECRET, 5).unwrap().token, SECRET).unwrap();
        let b = decode_token(&issue_token(id, SECRET, 5).unwrap().token, SECRET).unwrap();
        assert_ne!(a.jti, b.jti);
    }
}
