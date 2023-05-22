-- Your SQL goes here
CREATE TABLE IF NOT EXISTS chapter_pages(
    id UUID PRIMARY KEY,
    number INTEGER NOT NULL,
    path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    comic_id UUID NOT NULL,
    chapter_id UUID NOT NULL,
    user_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,

  FOREIGN KEY(comic_id)
    REFERENCES comics(id)
    ON DELETE CASCADE
    ON UPDATE CASCADE,

    FOREIGN KEY(chapter_id)
        REFERENCES comic_chapters(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    UNIQUE (chapter_id, number)
);