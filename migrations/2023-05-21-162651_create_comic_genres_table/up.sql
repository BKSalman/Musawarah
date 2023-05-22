-- Your SQL goes here
CREATE TABLE IF NOT EXISTS comic_genres (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

INSERT INTO comic_genres(name, created_at)
    VALUES('Action', now());
INSERT INTO comic_genres(name, created_at)
    VALUES('Adventure', now());
INSERT INTO comic_genres(name, created_at)
    VALUES('Romance', now());
-- TODO: add more initial genres
