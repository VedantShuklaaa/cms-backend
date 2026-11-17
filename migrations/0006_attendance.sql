CREATE TABLE attendance (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id      UUID        NOT NULL REFERENCES subjects (id) ON DELETE RESTRICT,
    student_id      UUID        NOT NULL REFERENCES students (id) ON DELETE RESTRICT,
    marked_by       UUID        REFERENCES faculty (id) ON DELETE SET NULL,
    attendance_date DATE        NOT NULL,
    status          TEXT        NOT NULL CHECK (status IN ('PRESENT', 'ABSENT', 'LATE', 'EXCUSED')),
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX attendance_subject_student_date_unique
    ON attendance (subject_id, student_id, attendance_date);
CREATE INDEX attendance_student_date_idx ON attendance (student_id, attendance_date);
CREATE INDEX attendance_subject_date_idx ON attendance (subject_id, attendance_date);

CREATE TRIGGER attendance_set_updated_at
    BEFORE UPDATE ON attendance
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
