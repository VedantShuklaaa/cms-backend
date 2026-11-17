use axum::{extract::Request, middleware::Next, response::Response};

use crate::{
    errors::{AppError, AppResult},
    models::{auth::AuthUser, user::Role},
};

/// Service-level role check. Resource ownership checks are added per feature in Phase 2.
pub fn ensure_role(user: &AuthUser, allowed: &[Role]) -> AppResult<()> {
    if allowed.contains(&user.role) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "You do not have permission to perform this action".into(),
        ))
    }
}

// Route middleware. Must sit INSIDE `require_auth` (add require_auth's route_layer last).
pub async fn admin_only(user: AuthUser, req: Request, next: Next) -> Result<Response, AppError> {
    ensure_role(&user, &[Role::Admin])?;
    Ok(next.run(req).await)
}

pub async fn staff_only(user: AuthUser, req: Request, next: Next) -> Result<Response, AppError> {
    ensure_role(&user, &[Role::Admin, Role::Faculty])?;
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn user(role: Role) -> AuthUser {
        AuthUser {
            id: Uuid::new_v4(),
            email: "u@test.dev".into(),
            role,
            jti: Uuid::new_v4(),
            expires_at: Utc::now(),
        }
    }

    #[test]
    fn allows_listed_roles() {
        assert!(ensure_role(&user(Role::Admin), &[Role::Admin]).is_ok());
        assert!(ensure_role(&user(Role::Faculty), &[Role::Admin, Role::Faculty]).is_ok());
    }

    #[test]
    fn denies_unlisted_roles() {
        assert!(matches!(
            ensure_role(&user(Role::Student), &[Role::Admin, Role::Faculty]),
            Err(AppError::Forbidden(_))
        ));
        assert!(matches!(
            ensure_role(&user(Role::Faculty), &[Role::Admin]),
            Err(AppError::Forbidden(_))
        ));
    }

    #[test]
    fn empty_allow_list_denies_everyone() {
        assert!(ensure_role(&user(Role::Admin), &[]).is_err());
    }
}
