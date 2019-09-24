-- Your SQL goes here
CREATE TABLE users_sessions (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    bio TEXT,
    image TEXT,
    password TEXT NOT NULL
)