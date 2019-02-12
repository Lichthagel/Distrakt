use crate::schema::{episodes, movies, notifications, shows, users};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use trakt::models::{
    FullCalendarMovie as TraktCalendarMovie, FullEpisode as TraktEpisode, FullShow as TraktShow,
};

#[derive(Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    pub discord_id: i64,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: NaiveDateTime,
    pub slug: String,
    pub username: String,
    pub name: Option<String>,
    pub private: bool,
    pub vip: Option<bool>,
    pub cover_image: Option<String>,
    pub avatar: Option<String>,
    pub joined_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Insertable)]
#[table_name = "shows"]
pub struct Show {
    pub title: String,
    pub year: Option<i32>,
    pub slug: Option<String>,
    pub trakt_id: Option<i64>,
    pub tvdb_id: Option<i64>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub tvrage_id: Option<i64>,
    pub overview: Option<String>,
    pub runtime: Option<i32>,
    pub trailer: Option<String>,
    pub homepage: Option<String>,
}

impl From<TraktShow> for Show {
    fn from(show: TraktShow) -> Self {
        Self {
            title: show.title,
            year: show.year.map(|i| i as i32),
            slug: show.ids.slug,
            trakt_id: show.ids.trakt.map(|i| i as i64),
            tvdb_id: show.ids.tvdb.map(|i| i as i64),
            imdb_id: show.ids.imdb,
            tmdb_id: show.ids.tmdb.map(|i| i as i64),
            tvrage_id: show.ids.tvrage.map(|i| i as i64),
            overview: show.overview,
            runtime: Some(show.runtime as i32),
            trailer: show.trailer,
            homepage: show.homepage,
        }
    }
}

#[derive(Queryable, Insertable)]
#[table_name = "episodes"]
pub struct Episode {
    pub trakt_id: i64,
    pub title: String,
    pub show_slug: String,
    pub season_num: i32,
    pub episode_num: i32,
    pub first_aired: Option<NaiveDateTime>,
    pub slug: Option<String>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub tvdb_id: Option<i64>,
    pub tvrage_id: Option<i64>,
    pub overview: Option<String>,
    pub runtime: Option<i32>,
}

impl From<(TraktEpisode, DateTime<Utc>, String)> for Episode {
    fn from((episode, first_aired, show_slug): (TraktEpisode, DateTime<Utc>, String)) -> Self {
        Self {
            title: episode.title.unwrap(),
            show_slug,
            season_num: episode.season as i32,
            episode_num: episode.number as i32,
            first_aired: Some(first_aired.naive_utc()),
            slug: episode.ids.slug,
            trakt_id: episode.ids.trakt.unwrap() as i64,
            tvdb_id: episode.ids.tvdb.map(|i| i as i64),
            imdb_id: episode.ids.imdb,
            tmdb_id: episode.ids.tmdb.map(|i| i as i64),
            tvrage_id: episode.ids.tvrage.map(|i| i as i64),
            overview: episode.overview,
            runtime: Some(episode.runtime as i32),
        }
    }
}

#[derive(Queryable, Insertable)]
#[table_name = "movies"]
pub struct Movie {
    pub slug: String,
    pub released: Option<NaiveDate>,
    pub title: String,
    pub year: Option<i32>,
    pub trakt_id: i64,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub tvdb_id: Option<i64>,
    pub tvrage_id: Option<i64>,
    pub overview: Option<String>,
    pub runtime: Option<i32>,
    pub trailer: Option<String>,
    pub homepage: Option<String>,
}

impl From<TraktCalendarMovie> for Movie {
    fn from(movie: TraktCalendarMovie) -> Self {
        Self {
            title: movie.movie.title,
            slug: movie.movie.ids.slug.unwrap(),
            trakt_id: movie.movie.ids.trakt.unwrap() as i64,
            tvdb_id: movie.movie.ids.tvdb.map(|i| i as i64),
            imdb_id: movie.movie.ids.imdb,
            tmdb_id: movie.movie.ids.tmdb.map(|i| i as i64),
            tvrage_id: movie.movie.ids.tvrage.map(|i| i as i64),
            overview: Some(movie.movie.overview),
            runtime: Some(movie.movie.runtime as i32),
            trailer: movie.movie.trailer,
            released: Some(movie.released),
            year: movie.movie.year.map(|i| i as i32),
            homepage: movie.movie.homepage,
        }
    }
}

#[derive(Queryable, Insertable)]
#[table_name = "notifications"]
pub struct Notification {
    pub channel: i64,
    pub trakt_id: i64,
}

impl Notification {
    pub fn new(channel: u64, trakt_id: u64) -> Self {
        Self {
            channel: channel as i64,
            trakt_id: trakt_id as i64,
        }
    }
}
