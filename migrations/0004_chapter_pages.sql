-- Add migration script here
CREATE TABLE IF NOT EXISTS chapter_pages (
    id uuid PRIMARY KEY NOT NULL,
    number integer NOT NULL,
    path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    chapter_id uuid NOT NULL REFERENCES chapters(id),
    comic_id uuid NOT NULL REFERENCES comics(id),
    author_id uuid NOT NULL REFERENCES users(id),

    UNIQUE (chapter_id, number)
);

