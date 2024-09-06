// @generated automatically by Diesel CLI.

diesel::table! {
    authors (id) {
        id -> Integer,
        hash -> Text,
    }
}

diesel::table! {
    confession (id) {
        id -> Integer,
        guild_id -> Text,
        message_id -> Text,
        content -> Text,
        author -> Integer,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    delete_votes (id) {
        id -> Integer,
        confession_id -> Integer,
        author_id -> Integer,
        vote_type -> Text,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    guild (guild_id) {
        guild_id -> Text,
        confession_channel_id -> Nullable<Text>,
        config -> Text,
        timestamp -> Timestamp,
    }
}

diesel::joinable!(confession -> authors (author));
diesel::joinable!(confession -> guild (guild_id));
diesel::joinable!(delete_votes -> authors (author_id));
diesel::joinable!(delete_votes -> confession (confession_id));

diesel::allow_tables_to_appear_in_same_query!(
    authors,
    confession,
    delete_votes,
    guild,
);
