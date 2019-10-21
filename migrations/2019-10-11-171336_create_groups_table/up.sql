-- Your SQL goes here
CREATE TABLE groups (
    id INT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    admin INTEGER NOT NULL REFERENCES users ON DELETE CASCADE
)