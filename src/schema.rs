table! {
    cards (id) {
        id -> Int4,
        phrase -> Text,
        images -> Array<Text>,
    }
}

table! {
    user_auths (id) {
        id -> Int4,
        user_id -> Int4,
        password_hash -> Text,
        salt -> Text,
    }
}

table! {
    user_cards (id) {
        id -> Int4,
        user_id -> Int4,
        phrase -> Text,
        images -> Array<Text>,
        categories -> Array<Text>,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    cards,
    user_auths,
    user_cards,
    users,
);
