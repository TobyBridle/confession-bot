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
        deleted -> Integer,
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

diesel::table! {
    replies (id) {
        id -> Integer,
        guild_id -> Text,
        original_confession_id -> Integer,
        message_id -> Text,
        content -> Text,
        author -> Integer,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    schedule (id) {
        id -> Integer,
        guild_id -> Text,
        victim_id -> Text,
        ends_at -> Integer,
        start_at -> Integer,
    }
}

diesel::joinable!(confession -> authors (author));
diesel::joinable!(confession -> guild (guild_id));
diesel::joinable!(delete_votes -> authors (author_id));
diesel::joinable!(delete_votes -> confession (confession_id));
diesel::joinable!(replies -> authors (author));
diesel::joinable!(replies -> confession (original_confession_id));
diesel::joinable!(replies -> guild (guild_id));

diesel::allow_tables_to_appear_in_same_query!(
    authors,
    confession,
    delete_votes,
    guild,
    replies,
    schedule,
);
