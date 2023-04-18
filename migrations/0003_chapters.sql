-- Add migration script here
CREATE TABLE IF NOT EXISTS chapters (
    id uuid PRIMARY KEY NOT NULL,
    number integer NOT NULL,
    description TEXT,
    comic_id uuid NOT NULL REFERENCES comics(id),
    author_id uuid NOT NULL REFERENCES users(id),

    UNIQUE (comic_id, number)
);

