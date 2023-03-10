-- Add migration script here
CREATE TABLE IF NOT EXISTS images (
    id uuid PRIMARY KEY NOT NULL,
    path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    post_id uuid NOT NULL REFERENCES posts(id),
    user_id uuid NOT NULL REFERENCES users(id)
);
