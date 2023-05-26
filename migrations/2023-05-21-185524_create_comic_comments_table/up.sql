-- Your SQL goes here
CREATE TABLE IF NOT EXISTS comic_comments (
    id UUID PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    comic_id UUID NOT NULL,
    user_id UUID NOT NULL,

    FOREIGN KEY(comic_id)
        REFERENCES comics(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);