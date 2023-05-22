-- Your SQL goes here
CREATE TABLE IF NOT EXISTS comics (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    user_id UUID NOT NULL,

    FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);