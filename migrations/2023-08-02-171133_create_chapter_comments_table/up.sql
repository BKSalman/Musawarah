-- Your SQL goes here
CREATE TABLE IF NOT EXISTS chapter_comments (
    id UUID PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    chapter_id UUID NOT NULL,
    user_id UUID NOT NULL,

    FOREIGN KEY(chapter_id)
        REFERENCES comic_chapters(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);