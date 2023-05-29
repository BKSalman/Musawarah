-- Your SQL goes here
CREATE TABLE IF NOT EXISTS comics (
    id UUID PRIMARY KEY,
    title TEXT UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    rating FLOAT,
    is_visible BOOLEAN NOT NULL,
    published_at TIMESTAMPTZ,
    poster_path TEXT,
    poster_content_type TEXT,
    user_id UUID NOT NULL,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);