-- Add migration script here
CREATE TABLE IF NOT EXISTS posts (
    id uuid PRIMARY KEY NOT NULL,
    author_id uuid NOT NULL REFERENCES users(id),
    title VARCHAR ( 150 ) NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);
