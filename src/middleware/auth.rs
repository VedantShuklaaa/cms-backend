use axum::{
    extract::{FromRequestParts, Request, State},
    http::{HeaderMap, header, request::Parts},
    middleware::Next,
    response::Response,
};

use crate::{
    errors::{AppError, AppResult},
    models::auth::AuthUser,
    services::auth as auth_service,
    state::AppState,
};

/// Validates the bearer token and stores the AuthUser in request extensions.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = bearer_token(req.headers())?;
    let user = auth_service::authenticate(&state, &token).await?;
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

fn bearer_token(headers: &HeaderMap) -> AppResult<String> {
    let value = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid Authorization header".into()))?;

    match value.split_once(' ') {
        Some((scheme, token))
            if scheme.eq_ignore_ascii_case("bearer") && !token.trim().is_empty() =>
        {
            Ok(token.trim().to_string())
        }
        _ => Err(AppError::Unauthorized(
            "Authorization header must be `Bearer <token>`".into(),
        )),
    }
}

/// Handlers take `AuthUser` as an argument. Requires `require_auth` to have run first.
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Authentication required".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(v: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::AUTHORIZATION, HeaderValue::from_str(v).unwrap());
        h
    }

    #[test]
    fn parses_bearer_case_insensitively() {
        assert_eq!(bearer_token(&headers("Bearer abc")).unwrap(), "abc");
        assert_eq!(bearer_token(&headers("bearer abc")).unwrap(), "abc");
    }

    #[test]
    fn rejects_missing_or_malformed() {
        assert!(bearer_token(&HeaderMap::new()).is_err());
        assert!(bearer_token(&headers("Basic abc")).is_err());
        assert!(bearer_token(&headers("Bearer")).is_err());
        assert!(bearer_token(&headers("Bearer ")).is_err());
    }
}
