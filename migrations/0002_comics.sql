-- Add migration script here
CREATE TABLE IF NOT EXISTS comics (
    id uuid PRIMARY KEY NOT NULL,
    author_id uuid NOT NULL REFERENCES users(id),
    title TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);
