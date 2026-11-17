CREATE TABLE exams (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID        NOT NULL REFERENCES subjects (id) ON DELETE RESTRICT,
    name       TEXT        NOT NULL,
    exam_type  TEXT        NOT NULL CHECK (exam_type IN ('QUIZ', 'MIDTERM', 'FINAL', 'PRACTICAL')),
    exam_date  DATE        NOT NULL,
    max_marks  SMALLINT    NOT NULL CHECK (max_marks BETWEEN 1 AND 1000),
    is_active  BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX exams_subject_name_unique ON exams (subject_id, lower(name));
CREATE INDEX exams_subject_idx ON exams (subject_id);

CREATE TRIGGER exams_set_updated_at
    BEFORE UPDATE ON exams
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE results (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exam_id        UUID        NOT NULL REFERENCES exams (id) ON DELETE RESTRICT,
    student_id     UUID        NOT NULL REFERENCES students (id) ON DELETE RESTRICT,
    marks_obtained SMALLINT    NOT NULL CHECK (marks_obtained >= 0),
    grade          TEXT,
    recorded_by    UUID        REFERENCES faculty (id) ON DELETE SET NULL,
    is_active      BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX results_exam_student_unique ON results (exam_id, student_id);
CREATE INDEX results_student_idx ON results (student_id);

CREATE TRIGGER results_set_updated_at
    BEFORE UPDATE ON results
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
