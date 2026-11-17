CREATE TABLE subjects (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id   UUID        NOT NULL REFERENCES courses (id) ON DELETE RESTRICT,
    name        TEXT        NOT NULL,
    code        TEXT        NOT NULL,
    semester    SMALLINT    NOT NULL CHECK (semester BETWEEN 1 AND 12),
    credits     SMALLINT    NOT NULL CHECK (credits BETWEEN 1 AND 10),
    description TEXT,
    is_active   BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX subjects_code_unique ON subjects (code);
CREATE UNIQUE INDEX subjects_course_name_unique ON subjects (course_id, lower(name));
CREATE INDEX subjects_course_semester_idx ON subjects (course_id, semester);

CREATE TRIGGER subjects_set_updated_at
    BEFORE UPDATE ON subjects
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE faculty_subjects (
    faculty_id  UUID        NOT NULL REFERENCES faculty (id) ON DELETE CASCADE,
    subject_id  UUID        NOT NULL REFERENCES subjects (id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (faculty_id, subject_id)
);

CREATE INDEX faculty_subjects_subject_idx ON faculty_subjects (subject_id);
