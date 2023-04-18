-- Add migration script here
CREATE TABLE IF NOT EXISTS comments (
    parent_comment_id uuid NOT NULL REFERENCES comments(id),
    child_comment_id uuid NOT NULL REFERENCES comments(id)
);

