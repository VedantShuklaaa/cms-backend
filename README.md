# CMS Backend

College Management System API. Rust, Axum, SQLx, PostgreSQL.

## Status

* \[x] Phase 1 / Step 1: config, Postgres (Docker), auto-migrations, server, CORS, health, error format
* \[ ] Phase 1 / Step 2: authentication and RBAC
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
