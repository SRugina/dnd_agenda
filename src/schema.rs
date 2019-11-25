table! {
    groups (id) {
        id -> Int4,
        slug -> Text,
        name -> Text,
        description -> Text,
        image -> Nullable<Text>,
        admin -> Int4,
    }
}

table! {
    groups_users (group_id, user_id) {
        group_id -> Int4,
        user_id -> Int4,
        admin_accepted -> Bool,
        user_accepted -> Bool,
    }
}

table! {
    sessions (id) {
        id -> Int4,
        slug -> Text,
        title -> Text,
        description -> Text,
        dm -> Int4,
        session_date -> Timestamptz,
        colour -> Text,
        image -> Nullable<Text>,
        group_id -> Int4,
    }
}

table! {
    sessions_guests (session_id, guest_id) {
        session_id -> Int4,
        guest_id -> Int4,
        guest_name -> Text,
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

joinable!(groups -> users (admin));
joinable!(groups_users -> groups (group_id));
joinable!(groups_users -> users (user_id));
joinable!(sessions -> groups (group_id));
joinable!(sessions -> users (dm));
joinable!(sessions_guests -> sessions (session_id));
joinable!(sessions_users -> sessions (session_id));
joinable!(sessions_users -> users (user_id));

allow_tables_to_appear_in_same_query!(
    groups,
    groups_users,
    sessions,
    sessions_guests,
    sessions_users,
    users,
);
