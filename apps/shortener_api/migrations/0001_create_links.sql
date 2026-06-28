CREATE TABLE links (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    short_code   TEXT NOT NULL UNIQUE,
    original_url TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
