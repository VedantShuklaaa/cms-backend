# CMS Backend

College Management System API. Rust, Axum, SQLx, PostgreSQL.

## Status

* \[x] Phase 1: config, Postgres (Docker), auto-migrations, server, CORS, health, errors, auth, RBAC
* \[x] Phase 2 / Step 1: shared helpers, departments, courses
* \[ ] Phase 2: faculty, subjects, students, attendance, exams/results, notices, fees, assignments, reports
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

## Shared API conventions

* List endpoints accept `?page=1&per_page=20` (per\_page max 100) and return `{ items, page, per_page, total }` inside `data`.
* Extractors `ApiJson`, `ApiQuery`, `ApiPath` return the standard error envelope on bad input.
* DELETE is a soft deactivation (`is_active = false`) and returns 200 with the envelope.
* Inactive records are hidden from non-admins; admins can pass `?include_inactive=true` on lists.
* `search` does a case-insensitive match; LIKE wildcards in user input are escaped.

## Departments and courses

* `GET /api/v1/departments`, `GET /api/v1/departments/{id}`: any authenticated user. Filter: `search`.
* `POST`, `PUT /{id}`, `DELETE /{id}`: admin only. Body fields: `name`, `code`, `description`; PUT also accepts `is_active`.
* A department with active courses cannot be deactivated (409).
* `GET /api/v1/courses`, `GET /api/v1/courses/{id}`: any authenticated user. Filters: `search`, `department_id`.
* `POST`, `PUT /{id}`, `DELETE /{id}`: admin only. Body: `department_id`, `name`, `code`, `duration_semesters` (1 to 12), `description`.
* Codes are uppercased and unique. Courses must belong to an active department.
