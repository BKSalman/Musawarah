-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    username VARCHAR ( 150 ) UNIQUE NOT NULL,
    displayname varchar (150) NOT NULL,
    password VARCHAR ( 150 ) NOT NULL,
    email VARCHAR ( 500 ) UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    last_login TIMESTAMP 
);
