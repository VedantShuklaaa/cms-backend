CREATE TABLE assignments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id  UUID        NOT NULL REFERENCES subjects (id) ON DELETE RESTRICT,
    faculty_id  UUID        NOT NULL REFERENCES faculty (id) ON DELETE RESTRICT,
    title       TEXT        NOT NULL,
    description TEXT        NOT NULL,
    due_date    DATE        NOT NULL,
    max_marks   SMALLINT    CHECK (max_marks BETWEEN 1 AND 1000),
    is_active   BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX assignments_subject_title_unique ON assignments (subject_id, lower(title));
CREATE INDEX assignments_subject_idx ON assignments (subject_id);
CREATE INDEX assignments_faculty_idx ON assignments (faculty_id);

CREATE TRIGGER assignments_set_updated_at
    BEFORE UPDATE ON assignments
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE assignment_submissions (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id  UUID        NOT NULL REFERENCES assignments (id) ON DELETE CASCADE,
    student_id     UUID        NOT NULL REFERENCES students (id) ON DELETE RESTRICT,
    content        TEXT        NOT NULL,
    submitted_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    marks_obtained SMALLINT    CHECK (marks_obtained >= 0),
    graded_by      UUID        REFERENCES faculty (id) ON DELETE SET NULL,
    graded_at      TIMESTAMPTZ,
    is_active      BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX assignment_submissions_unique
    ON assignment_submissions (assignment_id, student_id);
CREATE INDEX assignment_submissions_student_idx ON assignment_submissions (student_id);

CREATE TRIGGER assignment_submissions_set_updated_at
    BEFORE UPDATE ON assignment_submissions
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
