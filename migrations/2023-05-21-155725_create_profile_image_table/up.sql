-- Your SQL goes here
CREATE TABLE IF NOT EXISTS profile_images (
    id UUID PRIMARY KEY,
    path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    user_id UUID NOT NULL,
    updated_at TIMESTAMPTZ,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);
