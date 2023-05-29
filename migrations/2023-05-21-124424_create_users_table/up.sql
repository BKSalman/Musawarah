-- Your SQL goes here

CREATE TYPE UserRole AS ENUM (
    'admin', 'staff', 'user'
);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    first_name TEXT,
    last_name TEXT,
    username TEXT NOT NULL,
    displayname TEXT NOT NULL,
    email TEXT NOT NULL,
    phone_number TEXT,
    bio TEXT,
    password TEXT NOT NULL,
    role UserRole NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    last_login TIMESTAMPTZ
)