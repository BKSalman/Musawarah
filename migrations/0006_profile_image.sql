-- Add migration script here
CREATE TABLE IF NOT EXISTS profile_images (
    id uuid PRIMARY KEY NOT NULL,
    path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    user_id uuid NOT NULL REFERENCES users(id)
);

