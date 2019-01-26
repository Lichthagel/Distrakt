table! {
    users (discord_id) {
        discord_id -> BigInt,
        access_token -> Text,
        refresh_token -> Text,
        expires -> Timestamp,
    }
}
