CREATE TABLE departments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT        NOT NULL,
    code        TEXT        NOT NULL,
    description TEXT,
    is_active   BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX departments_code_unique ON departments (code);
CREATE UNIQUE INDEX departments_name_unique ON departments (lower(name));

CREATE TRIGGER departments_set_updated_at
    BEFORE UPDATE ON departments
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE courses (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    department_id      UUID        NOT NULL REFERENCES departments (id) ON DELETE RESTRICT,
    name               TEXT        NOT NULL,
    code               TEXT        NOT NULL,
    duration_semesters SMALLINT    NOT NULL CHECK (duration_semesters BETWEEN 1 AND 12),
    description        TEXT,
    is_active          BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX courses_code_unique ON courses (code);
CREATE INDEX courses_department_idx ON courses (department_id);

CREATE TRIGGER courses_set_updated_at
    BEFORE UPDATE ON courses
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();