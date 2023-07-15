-- Your SQL goes here
CREATE TABLE IF NOT EXISTS comic_chapters (
    id UUID PRIMARY KEY,
    title TEXT,
    description TEXT,
    number INTEGER NOT NULL,
    poster_path TEXT,
    poster_content_type TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    is_visible BOOLEAN NOT NULL,
    user_id UUID NOT NULL,
    comic_id UUID NOT NULL,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    FOREIGN KEY(comic_id)
        REFERENCES comics(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    UNIQUE (comic_id, number)
);