table! {
    sessions (id) {
        id -> Int4,
        slug -> Text,
        title -> Text,
        description -> Text,
        dm -> Int4,
        session_date -> Timestamptz,
        colour -> Text,
    }
}

table! {
    sessions_users (session_id, user_id) {
        session_id -> Int4,
        user_id -> Int4,
        dm_accepted -> Bool,
        user_accepted -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
        email -> Text,
        bio -> Nullable<Text>,
        image -> Nullable<Text>,
        password -> Text,
    }
}

joinable!(sessions -> users (dm));
joinable!(sessions_users -> sessions (session_id));
joinable!(sessions_users -> users (user_id));

allow_tables_to_appear_in_same_query!(
    sessions,
    sessions_users,
    users,
);
