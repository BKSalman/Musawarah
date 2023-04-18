-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id uuid PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    displayname TEXT NOT NULL,
    password TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    last_login TIMESTAMP
);
