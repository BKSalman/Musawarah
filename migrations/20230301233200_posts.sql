-- Add migration script here
CREATE TABLE IF NOT EXISTS posts (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    author_id uuid NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),

    CONSTRAINT fk_author FOREIGN KEY(author_id) REFERENCES users(id)
);
