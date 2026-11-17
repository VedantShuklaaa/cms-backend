CREATE TABLE faculty (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID        NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    department_id UUID        NOT NULL REFERENCES departments (id) ON DELETE RESTRICT,
    employee_id   TEXT        NOT NULL,
    first_name    TEXT        NOT NULL,
    last_name     TEXT        NOT NULL,
    phone         TEXT,
    designation   TEXT        NOT NULL,
    joined_on     DATE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX faculty_user_unique ON faculty (user_id);
CREATE UNIQUE INDEX faculty_employee_id_unique ON faculty (employee_id);
CREATE INDEX faculty_department_idx ON faculty (department_id);

CREATE TRIGGER faculty_set_updated_at
    BEFORE UPDATE ON faculty
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE students (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID        NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    course_id        UUID        NOT NULL REFERENCES courses (id) ON DELETE RESTRICT,
    enrollment_no    TEXT        NOT NULL,
    first_name       TEXT        NOT NULL,
    last_name        TEXT        NOT NULL,
    phone            TEXT,
    date_of_birth    DATE,
    current_semester SMALLINT    NOT NULL CHECK (current_semester BETWEEN 1 AND 12),
    admission_year   SMALLINT    NOT NULL CHECK (admission_year BETWEEN 1950 AND 2200),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX students_user_unique ON students (user_id);
CREATE UNIQUE INDEX students_enrollment_no_unique ON students (enrollment_no);
CREATE INDEX students_course_semester_idx ON students (course_id, current_semester);

CREATE TRIGGER students_set_updated_at
    BEFORE UPDATE ON students
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();