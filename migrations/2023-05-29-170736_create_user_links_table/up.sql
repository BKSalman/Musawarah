-- Your SQL goes here
CREATE TABLE IF NOT EXISTS user_links(
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    link TEXT NOT NULL,
    user_id UUID NOT NULL
);