# CMS Backend

College Management System API. Rust, Axum, SQLx, PostgreSQL.

## Status

* \[x] Phase 1 / Step 1: config, Postgres (Docker), auto-migrations, server, CORS, health, error format
* \[x] Phase 1 / Step 2: authentication (JWT + Argon2id) and RBAC
* \[ ] Phase 2: feature APIs
* \[ ] Phase 3: Next.js frontend

## Local setup

```bash
cp .env.example .env          # then edit the password and JWT_SECRET
openssl rand -hex 32          # paste into JWT_SECRET
docker compose up -d          # start Postgres (container: cms-db)
cargo run                     # runs pending migrations, then starts the server
```

## Database

* Start: `docker compose up -d`
* Stop (keeps data): `docker compose down`
* Logs: `docker compose logs -f db`
* Status: `docker compose ps`
* Wipe all data (destructive): `docker compose down -v`
* Note: `POSTGRES_PASSWORD` only applies on first volume init. Changing it later needs `down -v` or an `ALTER USER`.

## Migrations

* Files in `migrations/`, named `NNNN_description.sql`, applied automatically on startup.
* Applied migrations are tracked in `_sqlx_migrations` and never re-run.
* Never edit an applied migration. Add a new corrective migration instead.
* If a migration fails, the app exits before binding the port.

## Health check

```bash
curl http://127.0.0.1:8080/api/v1/health
```

## Response format

Success: `{ "success": true, "message": "...", "data": {} }`
Error: `{ "success": false, "message": "...", "error": "..." }`

## Config

See `.env.example`. `FRONTEND_URL` is a comma-separated list of allowed CORS origins (no wildcards).

## Authentication

* `POST /api/v1/auth/login` with `{ "email", "password" }` returns `{ access_token, token_type, expires_at, user }`.
* Send `Authorization: Bearer <token>` on every private endpoint.
* `POST /api/v1/auth/logout` revokes the current token (stored in `revoked_tokens`).
* `GET /api/v1/auth/me` returns the current user.
* `POST /api/v1/auth/change-password` with `{ "current_password", "new_password" }`. All previously issued tokens stop working; a fresh token is returned.
* Tokens are HS256 JWTs (`sub`, `jti`, `iat`, `exp`). The role is always read from the database, never from the token.
* Passwords: Argon2id, 8 to 128 characters. Hashes are never returned by the API.
* No public registration. The first admin is created at startup from `ADMIN_EMAIL` / `ADMIN_PASSWORD` when no admin exists. Remove both variables from `.env` afterwards.

## RBAC

* `middleware::auth::require_auth` authenticates; handlers receive `AuthUser`.
* `middleware::rbac::{admin_only, staff_only}` guard route groups; `ensure_role()` is for service-level checks.
* Add `require_auth` as the LAST `route_layer` so it runs before the role layer.
* Ownership checks (student sees only own data, faculty only assigned subjects) live in services, per feature.
* `GET /api/v1/auth/admin-ping` is a temporary probe, to be removed in Phase 2.
