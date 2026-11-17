CREATE TABLE fee_structures (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id     UUID        NOT NULL REFERENCES courses (id) ON DELETE RESTRICT,
    academic_year SMALLINT    NOT NULL CHECK (academic_year BETWEEN 1950 AND 2200),
    name          TEXT        NOT NULL,
    amount        BIGINT      NOT NULL CHECK (amount > 0),
    due_date      DATE        NOT NULL,
    is_active     BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX fee_structures_course_year_name_unique
    ON fee_structures (course_id, academic_year, lower(name));
CREATE INDEX fee_structures_course_idx ON fee_structures (course_id, academic_year);

CREATE TRIGGER fee_structures_set_updated_at
    BEFORE UPDATE ON fee_structures
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE fee_payments (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fee_structure_id UUID        NOT NULL REFERENCES fee_structures (id) ON DELETE RESTRICT,
    student_id       UUID        NOT NULL REFERENCES students (id) ON DELETE RESTRICT,
    amount           BIGINT      NOT NULL CHECK (amount > 0),
    paid_on          DATE        NOT NULL,
    method           TEXT        NOT NULL CHECK (method IN ('CASH', 'CARD', 'UPI', 'BANK_TRANSFER')),
    reference        TEXT,
    recorded_by      UUID        REFERENCES users (id) ON DELETE SET NULL,
    is_active        BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX fee_payments_student_idx ON fee_payments (student_id);
CREATE INDEX fee_payments_structure_idx ON fee_payments (fee_structure_id);

CREATE TRIGGER fee_payments_set_updated_at
    BEFORE UPDATE ON fee_payments
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
