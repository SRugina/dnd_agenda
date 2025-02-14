-- Your SQL goes here
CREATE TABLE sessions_users (
    session_id INT NOT NULL REFERENCES sessions (id) ON UPDATE CASCADE ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    dm_accepted BOOLEAN NOT NULL DEFAULT FALSE,
    user_accepted BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT sessions_users_pkey PRIMARY KEY (session_id, user_id)
)