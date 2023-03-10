-- Add migration script here
CREATE TABLE IF NOT EXISTS comments (
    id uuid PRIMARY KEY NOT NULL,
    author_id uuid NOT NULL REFERENCES users(id),
    post_id uuid NOT NULL REFERENCES posts(id),
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);
