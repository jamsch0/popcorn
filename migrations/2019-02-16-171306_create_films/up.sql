CREATE TABLE films (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now_utc(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now_utc(),
    title TEXT NOT NULL,
    release_year INT NOT NULL,
    summary TEXT NOT NULL,
    runtime_mins INT NOT NULL
);

SELECT diesel_manage_updated_at('films');
