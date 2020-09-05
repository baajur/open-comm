table! {
    user_auths (id) {
        id -> Int4,
        user_id -> Int4,
        password_hash -> Text,
        salt -> Text,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
    }
}

joinable!(user_auths -> users (user_id));

allow_tables_to_appear_in_same_query!(
    user_auths,
    users,
);
