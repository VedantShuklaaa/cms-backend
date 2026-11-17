# CMS Backend

College Management System API. Rust, Axum, SQLx, PostgreSQL.

## Status

* \[x] Phase 1: config, Postgres (Docker), auto-migrations, server, CORS, health, errors, auth, RBAC
* \[x] Phase 2 / Step 1: shared helpers, departments, courses
* \[x] Phase 2 / Step 2: faculty and students, with account creation and ownership checks
* \[x] Phase 2 / Step 3: subjects and faculty assignment, attendance, exams/results, notices, fees, assignments, reports
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

## API specification

`openapi.yaml` documents every endpoint, request body, response envelope and schema. Generate frontend types from it rather than writing them by hand:

```bash
npx openapi-typescript openapi.yaml -o src/types/api.ts
```

The file at the repo root can also be opened in Swagger UI or Redoc. Keep it in sync when routes or payloads change.

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
* Ownership checks (students see only their own data; faculty only their own profile and assigned subjects) live in services, per feature.

## Shared API conventions

* List endpoints accept `?page=1&per_page=20` (per\_page max 100) and return `{ items, page, per_page, total }` inside `data`.
* Extractors `ApiJson`, `ApiQuery`, `ApiPath` return the standard error envelope on bad input.
* DELETE is a soft deactivation (`is_active = false`) and returns 200 with the envelope.
* Inactive records are hidden from non-admins; admins can pass `?include_inactive=true` on lists.
* `search` does a case-insensitive match; LIKE wildcards in user input are escaped.
* Report endpoints return aggregate arrays, not paginated lists.
* Fee amounts are whole currency units (`BIGINT`).

## Departments and courses

* `GET /api/v1/departments`, `GET /api/v1/departments/{id}`: any authenticated user. Filter: `search`.
* `POST`, `PUT /{id}`, `DELETE /{id}`: admin only. Body fields: `name`, `code`, `description`; PUT also accepts `is_active`.
* A department with active courses cannot be deactivated (409).
* `GET /api/v1/courses`, `GET /api/v1/courses/{id}`: any authenticated user. Filters: `search`, `department_id`.
* `POST`, `PUT /{id}`, `DELETE /{id}`: admin only. Body: `department_id`, `name`, `code`, `duration_semesters` (1 to 12), `description`.
* Codes are uppercased and unique. Courses must belong to an active department.

## Faculty

* `GET /api/v1/faculty`, `POST /api/v1/faculty`: admin only. List filters: `search` (name, employee\_id, email), `department_id`, `include_inactive`.
* `GET /api/v1/faculty/me`: the signed-in faculty member's own profile.
* `GET /api/v1/faculty/{id}`: admin, or a faculty member for their own record.
* `PUT /api/v1/faculty/{id}`, `DELETE /{id}`: admin only. PUT is a partial update and also accepts `is_active`.
* Create body: `email`, `password`, `department_id`, `employee_id`, `first_name`, `last_name`, `designation`; optional `phone`, `joined_on`.
* The user account and faculty profile are created in one transaction. Email and employee\_id are unique; the department must be active.
* DELETE deactivates the linked user account; the profile row is kept.

## Students

* `GET /api/v1/students`: admin and faculty. Filters: `search` (name, enrollment\_no, email), `department_id`, `course_id`, `semester`, `include_inactive` (admin only).
* `POST /api/v1/students`: admin only.
* `GET /api/v1/students/me`: the signed-in student's own profile.
* `GET /api/v1/students/{id}`: admin and faculty, or a student for their own record.
* `PUT /api/v1/students/{id}`, `DELETE /{id}`: admin only. PUT is a partial update and also accepts `is_active`.
* Create body: `email`, `password`, `course_id`, `enrollment_no`, `first_name`, `last_name`, `current_semester`, `admission_year`; optional `phone`, `date_of_birth`.
* `current_semester` must fit the course's `duration_semesters`; `admission_year` must be 1950 through next year; `date_of_birth` must be in the past.
* The course must be active. The account and profile are created in one transaction. Email and enrollment\_no are unique.
* DELETE deactivates the linked user account; the profile row is kept.

## Subjects

* `GET /api/v1/subjects`, `GET /api/v1/subjects/{id}`: any authenticated user. Filters: `search` (name, code), `course_id`, `department_id`, `semester`, `faculty_id` (subjects assigned to that faculty member); `include_inactive` for admins.
* `GET /api/v1/subjects/mine`: faculty and students only. Faculty get their assigned subjects; students get the active subjects in their course for their current semester. Supports `search` and pagination. Admins get 403.
* `POST /api/v1/subjects`, `PUT /api/v1/subjects/{id}`, `DELETE /{id}`: admin only. Body: `course_id`, `name`, `code`, `semester`, `credits` (1 to 10), `description`; PUT also accepts `is_active`.
* Codes are uppercased and unique. A name is unique within its course. `semester` must fit the course's `duration_semesters`, and the course must be active.
* Faculty assignment:
  * `GET /api/v1/subjects/{id}/faculty`: admin and faculty.
  * `POST /api/v1/subjects/{id}/faculty`: admin only, body `{ "faculty_id" }`. Repeating it is safe.
  * `DELETE /api/v1/subjects/{id}/faculty/{faculty_id}`: admin only.
* Faculty access to attendance, results and assignments is scoped to the subjects they are assigned to.

## Attendance

* `GET /api/v1/attendance`: admin and faculty. Faculty see only assigned subjects. Filters: `subject_id`, `student_id`, `status`, `date_from`, `date_to`; `include_inactive` for admins.
* `POST /api/v1/attendance`: admin and faculty. Body: `subject_id`, `student_id`, `attendance_date`, `status` (`PRESENT`, `ABSENT`, `LATE`, `EXCUSED`).
* Posting the same student, subject and date updates the existing record. The date cannot be in the future, and the student must be enrolled in the subject's course.
* `GET /api/v1/attendance/{id}`: admin, assigned faculty, or the owning student.
* `PUT /{id}`, `DELETE /{id}`: admin or assigned faculty. DELETE is a soft deactivation.
* `GET /api/v1/attendance/me`: student only. Same filters as the list, minus `student_id`.

## Exams and results

* `GET /api/v1/exams`, `GET /api/v1/exams/{id}`: any authenticated user. Filters: `search` (exam, subject, subject code), `subject_id`, `course_id`, `exam_type`; `include_inactive` for admins.
* `POST /api/v1/exams`, `PUT /api/v1/exams/{id}`, `DELETE /{id}`: admin only. Body: `subject_id`, `name`, `exam_type` (`QUIZ`, `MIDTERM`, `FINAL`, `PRACTICAL`), `exam_date`, `max_marks` (1 to 1000); PUT also accepts `is_active`.
* The subject must be active. Once results exist, the exam's subject cannot change and `max_marks` cannot drop below an existing result.
* `GET /api/v1/results`: admin and faculty. Faculty see only assigned subjects. Filters: `exam_id`, `subject_id`, `student_id`; `include_inactive` for admins.
* `GET /api/v1/results/me`: student only. Filters: `exam_id`, `subject_id`.
* `GET /api/v1/results/{id}`: admin, assigned faculty, or the owning student.
* `POST /api/v1/results`: admin and faculty. Body: `exam_id`, `student_id`, `marks_obtained`, optional `grade`. Posting the same exam and student updates the result.
* `marks_obtained` must be between 0 and the exam's `max_marks`; the student must be enrolled in the exam subject's course.
* `PUT /{id}`, `DELETE /{id}`: admin or assigned faculty. DELETE is a soft deactivation.

## Notices

* `GET /api/v1/notices`, `GET /api/v1/notices/{id}`: any authenticated user. Non-admins see only active, published, unexpired notices addressed to `ALL` or their own role.
* Admin list filters: `search` (title, body), `audience`, `include_inactive`, `include_expired`.
* `POST /api/v1/notices`, `PUT /api/v1/notices/{id}`, `DELETE /{id}`: admin only. Body: `title`, `body`, `audience` (`ALL`, `STUDENTS`, `FACULTY`, `ADMIN`; default `ALL`), optional `published_at`, optional `expires_at`; PUT also accepts `is_active`.
* `expires_at` must be after `published_at`. DELETE is a soft deactivation.

## Fees

* Fee structures:
  * `GET /api/v1/fees`, `GET /api/v1/fees/{id}`: authenticated users. Students are limited to their own course. Filters: `search`, `course_id`, `academic_year`; `include_inactive` for admins.
  * `POST /api/v1/fees`, `PUT /api/v1/fees/{id}`, `DELETE /{id}`: admin only. Body: `course_id`, `academic_year`, `name`, `amount`, `due_date`; PUT also accepts `is_active`.
  * A fee structure with active payments cannot be deactivated (409). Lowering `amount` below a student's recorded payments is rejected.
* Payments:
  * `GET /api/v1/fees/{id}/payments`: admin only. Filters: `student_id`; `include_inactive`.
  * `POST /api/v1/fees/{id}/payments`: admin only. Body: `student_id`, `amount`, `paid_on`, `method` (`CASH`, `CARD`, `UPI`, `BANK_TRANSFER`), optional `reference`.
  * The student must be enrolled in the fee's course; payments cannot exceed the outstanding balance; `paid_on` cannot be in the future.
  * `DELETE /api/v1/fees/{id}/payments/{payment_id}`: admin only, soft deactivation.
* Student self-service:
  * `GET /api/v1/fees/me`: fee statements with `paid_amount` and `balance`. Filter: `academic_year`.
  * `GET /api/v1/fees/me/payments`: the student's own payments. Filter: `academic_year`.

## Assignments and submissions

* `GET /api/v1/assignments`, `GET /api/v1/assignments/{id}`: any authenticated user. Admins see all; faculty see assigned subjects; students see their own course.
* List filters: `search` (title, description, subject), `subject_id`, `course_id`; `include_inactive` for admins.
* `POST /api/v1/assignments`: admin and faculty. Body: `subject_id`, `title`, `description`, `due_date`, optional `max_marks`.
* Faculty create under their own identity. Admins must pass `faculty_id`, and that faculty must be assigned to the subject.
* `PUT /api/v1/assignments/{id}`, `DELETE /{id}`: admin or faculty assigned to the subject. DELETE is a soft deactivation. `max_marks` cannot drop below a graded submission.
* Submissions:
  * `POST /api/v1/assignments/{id}/submissions`: student only. Body: `content`. Students can submit only for their own course and before `due_date`. Re-submitting replaces the content and clears the grade.
  * `GET /api/v1/assignments/{id}/submissions`: admin or assigned faculty. Filters: `student_id`; `include_inactive` for admins.
  * `PUT /api/v1/assignments/{id}/submissions/{submission_id}`: admin or assigned faculty. Body: optional `marks_obtained`, optional `is_active`. Marks must fit the assignment's `max_marks`.
  * `DELETE /api/v1/assignments/{id}/submissions/{submission_id}`: admin or assigned faculty, soft deactivation.
  * `GET /api/v1/submissions/me`: student only, the student's own submissions.

## Reports

* `GET /api/v1/reports/overview`: admin only. Active counts of departments, courses, subjects, faculty, students, notices and assignments.
* `GET /api/v1/reports/attendance`: admin and faculty (assigned subjects). Filters: `subject_id`, `course_id`, `department_id`, `date_from`, `date_to`. The percentage counts `PRESENT` and `LATE` as attended.
* `GET /api/v1/reports/results`: admin and faculty (assigned subjects). Filters: `exam_id`, `subject_id`, `course_id`. The pass rate uses 40% of `max_marks`.
* `GET /api/v1/reports/fees`: admin only. Filters: `course_id`, `academic_year`. Returns expected, collected and outstanding amounts per fee structure.
