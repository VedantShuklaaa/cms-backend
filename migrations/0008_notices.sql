CREATE TABLE notices (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title        TEXT        NOT NULL,
    body         TEXT        NOT NULL,
    audience     TEXT        NOT NULL DEFAULT 'ALL'
        CHECK (audience IN ('ALL', 'STUDENTS', 'FACULTY', 'ADMIN')),
    created_by   UUID        NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ,
    is_active    BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX notices_published_idx ON notices (published_at DESC);
CREATE INDEX notices_audience_idx ON notices (audience);

CREATE TRIGGER notices_set_updated_at
    BEFORE UPDATE ON notices
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
