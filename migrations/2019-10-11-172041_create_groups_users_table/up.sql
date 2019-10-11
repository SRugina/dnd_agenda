-- Your SQL goes here
CREATE TABLE groups_users (
    group_id INT NOT NULL REFERENCES groups (id) ON UPDATE CASCADE ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    admin_accepted BOOLEAN NOT NULL DEFAULT FALSE,
    user_accepted BOOLEAN NOT NULL DEFAULT FALSE,
    CONSTRAINT groups_users_pkey PRIMARY KEY (group_id, user_id)
)