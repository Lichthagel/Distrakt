table! {
    episodes (trakt_id) {
        trakt_id -> BigInt,
        title -> Text,
        show_slug -> Text,
        season_num -> Integer,
        episode_num -> Integer,
        first_aired -> Nullable<Timestamp>,
        slug -> Nullable<Text>,
        imdb_id -> Nullable<Text>,
        tmdb_id -> Nullable<BigInt>,
        tvdb_id -> Nullable<BigInt>,
        tvrage_id -> Nullable<BigInt>,
        overview -> Nullable<Text>,
        runtime -> Nullable<Integer>,
    }
}

table! {
    movies (slug) {
        slug -> Text,
        released -> Nullable<Date>,
        title -> Text,
        year -> Nullable<Integer>,
        trakt_id -> BigInt,
        imdb_id -> Nullable<Text>,
        tmdb_id -> Nullable<BigInt>,
        tvdb_id -> Nullable<BigInt>,
        tvrage_id -> Nullable<BigInt>,
        overview -> Nullable<Text>,
        runtime -> Nullable<Integer>,
        trailer -> Nullable<Text>,
        homepage -> Nullable<Text>,
    }
}

table! {
    notifications (channel, trakt_id) {
        channel -> BigInt,
        trakt_id -> BigInt,
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
        slug -> Text,
        title -> Text,
        year -> Nullable<Integer>,
        trakt_id -> Nullable<BigInt>,
        imdb_id -> Nullable<Text>,
        tmdb_id -> Nullable<BigInt>,
        tvdb_id -> Nullable<BigInt>,
        tvrage_id -> Nullable<BigInt>,
        overview -> Nullable<Text>,
        runtime -> Nullable<Integer>,
        trailer -> Nullable<Text>,
        homepage -> Nullable<Text>,
    }
}

table! {
    users (discord_id) {
        discord_id -> BigInt,
        access_token -> Text,
        refresh_token -> Text,
        expires -> Timestamp,
        slug -> Text,
        username -> Text,
        name -> Nullable<Text>,
        private -> Bool,
        vip -> Nullable<Bool>,
        cover_image -> Nullable<Text>,
        avatar -> Nullable<Text>,
        joined_at -> Nullable<Timestamp>,
    }
}

joinable!(episodes -> shows (show_slug));
joinable!(notify -> users (data));

allow_tables_to_appear_in_same_query!(
    episodes,
    movies,
    notifications,
    notify,
    shows,
    users,
);
