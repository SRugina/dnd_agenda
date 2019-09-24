-- Your SQL goes here
CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    dm INTEGER NOT NULL REFERENCES users ON DELETE CASCADE,
    session_date TEXT NOT NULL
)