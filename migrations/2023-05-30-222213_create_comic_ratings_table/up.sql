-- Your SQL goes here
CREATE TABLE IF NOT EXISTS comic_ratings (
    id UUID PRIMARY KEY,
    rating FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    -- rater
    user_id UUID NOT NULL,
    comic_id UUID NOT NULL,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    FOREIGN KEY(comic_id)
        REFERENCES comics(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);