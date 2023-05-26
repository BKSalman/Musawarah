-- Your SQL goes here
CREATE TABLE IF NOT EXISTS comic_chapters (
    id UUID PRIMARY KEY,
    title TEXT,
    description TEXT,
    number INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
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