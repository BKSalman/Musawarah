-- Your SQL goes here
CREATE TABLE IF NOT EXISTS chapter_ratings (
    id UUID PRIMARY KEY,
    rating FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    -- rater
    user_id UUID NOT NULL,
    chapter_id UUID NOT NULL,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    FOREIGN KEY(chapter_id)
        REFERENCES comic_chapters(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);