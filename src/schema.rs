table! {
    users (discord_id) {
        discord_id -> Integer,
        access_token -> Text,
        refresh_token -> Text,
        expires -> Date,
    }
}
