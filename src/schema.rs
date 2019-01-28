table! {
    episodes (trakt_id) {
        trakt_id -> Nullable<BigInt>,
        title -> Text,
        season_num -> Integer,
        episode_num -> Integer,
        first_aired -> Nullable<Timestamp>,
        slug -> Nullable<Text>,
        imdb_id -> Nullable<Text>,
        tmdb_id -> Nullable<BigInt>,
        tvdb_id -> Nullable<BigInt>,
        tvrage_id -> Nullable<BigInt>,
    }
}

table! {
    movies (slug) {
        slug -> Nullable<Text>,
        released -> Nullable<Timestamp>,
        title -> Text,
        year -> Nullable<Integer>,
        trakt_id -> Nullable<BigInt>,
        imdb_id -> Nullable<Text>,
        tmdb_id -> Nullable<BigInt>,
        tvdb_id -> Nullable<BigInt>,
        tvrage_id -> Nullable<BigInt>,
    }
}

table! {
    notify (channel, type_, data) {
        channel -> BigInt,
        #[sql_name = "type"]
        type_ -> Integer,
        data -> Nullable<BigInt>,
    }
}

table! {
    shows (slug) {
        slug -> Nullable<Text>,
        title -> Text,
        year -> Nullable<Integer>,
        trakt_id -> Nullable<BigInt>,
        imdb_id -> Nullable<Text>,
        tmdb_id -> Nullable<BigInt>,
        tvdb_id -> Nullable<BigInt>,
        tvrage_id -> Nullable<BigInt>,
    }
}

table! {
    users (discord_id) {
        discord_id -> BigInt,
        access_token -> Text,
        refresh_token -> Text,
        expires -> Timestamp,
    }
}

joinable!(notify -> users (data));

allow_tables_to_appear_in_same_query!(
    episodes,
    movies,
    notify,
    shows,
    users,
);
