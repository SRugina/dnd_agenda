table! {
    sessions (id) {
        id -> Int4,
        slug -> Text,
        title -> Text,
        description -> Text,
        dm -> Int4,
        date -> Text,
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

allow_tables_to_appear_in_same_query!(
    sessions,
    users,
);
