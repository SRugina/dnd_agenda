-- Your SQL goes here
CREATE TABLE groups_sessions (
    group_id INT NOT NULL REFERENCES groups (id) ON UPDATE CASCADE ON DELETE CASCADE,
    session_id INT NOT NULL REFERENCES sessions (id) ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT groups_sessions_pkey PRIMARY KEY (group_id, session_id)
)