use crate::{
    models::{Episode, Movie, Notification, Show},
    schema::{episodes, movies, notifications, notify, shows, users},
    wrappers::{Sqlite, Trakt},
};
use chrono::{DateTime, Utc};
use diesel::{
    prelude::{QueryResult, SqliteConnection},
    query_dsl::{QueryDsl, RunQueryDsl},
    ExpressionMethods, Insertable, NullableExpressionMethods,
};
use serenity::prelude::RwLock;
use std::{sync::Arc, thread, time::Duration};
use trakt::models::{
    CalendarMovie as TraktCalendarMovie, Episode as TraktEpisode, Show as TraktShow,
};
use typemap::ShareMap;

fn insert_show(conn: &SqliteConnection, show: TraktShow) -> QueryResult<usize> {
    Show::from(show).insert_into(shows::table).execute(conn)
}

fn insert_episode(
    conn: &SqliteConnection,
    episode: TraktEpisode,
    first_aired: DateTime<Utc>,
) -> QueryResult<usize> {
    Episode::from((episode, first_aired))
        .insert_into(episodes::table)
        .execute(conn)
}

fn insert_movie(conn: &SqliteConnection, movie: TraktCalendarMovie) -> QueryResult<usize> {
    Movie::from(movie).insert_into(movies::table).execute(conn)
}

fn insert_notification(conn: &SqliteConnection, channel: u64, trakt_id: u64) -> QueryResult<usize> {
    Notification::new(channel, trakt_id)
        .insert_into(notifications::table)
        .execute(conn)
}

pub fn sync_get_token(data: Arc<RwLock<ShareMap>>, _channel: u64, type_: u8, discord_id: u64) {
    data.read()
        .get::<Sqlite>()
        .ok_or(())
        .and_then(|sql| sql.lock().map_err(|_| ()))
        .and_then(|conn| {
            users::table
                .select(users::access_token)
                .filter(users::discord_id.eq(discord_id as i64))
                .limit(1)
                .load::<String>(&*conn)
                .map_err(|_| ())
        })
        .and_then(|res: Vec<String>| {
            if res.len() > 0 {
                sync(
                    data.clone(),
                    _channel,
                    type_,
                    discord_id,
                    res.get(0).unwrap(),
                );
                Ok(())
            } else {
                Err(())
            }
        })
        .ok();
}

pub fn sync(
    data: Arc<RwLock<ShareMap>>,
    _channel: u64,
    type_: u8,
    _discord_id: u64,
    access_token: &str,
) {
    if type_ % 2 == 0 {
        data.read().get::<Trakt>().map(|api| {
            // movies
            if type_ & 4 == 4 {
                api.calendar_my_movies(Utc::today(), 14, access_token)
                    .map(|res| {
                        for movie in res {
                            let id = movie.movie.ids.trakt.unwrap().clone();
                            data.read()
                                .get::<Sqlite>()
                                .ok_or(())
                                .and_then(|sql| sql.lock().map_err(|_| ()))
                                .and_then(|conn| {
                                    insert_movie(&*conn, movie)
                                        .and_then(|_| insert_notification(&*conn, _channel, id))
                                        .map_err(|_| ())
                                })
                                .ok();
                        }
                    })
                    .ok();
            }
            // shows
            if type_ & 8 == 8 {
                api.calendar_my_shows(Utc::today(), 14, access_token)
                    .map(|res| {
                        for show in res {
                            data.read()
                                .get::<Sqlite>()
                                .ok_or(())
                                .and_then(|conn| conn.lock().map_err(|_| ()))
                                .and_then(|conn| {
                                    insert_show(&*conn, show.show).ok();
                                    insert_episode(&*conn, show.episode, show.first_aired).ok();
                                    Ok(())
                                })
                                .ok();
                        }
                    })
                    .ok();
            }
        });
    }
}

pub fn sync_thread(data: Arc<RwLock<ShareMap>>) {
    thread::spawn(move || {
        loop {
            data.read()
                .get::<Sqlite>()
                .ok_or(())
                .and_then(|sql| sql.lock().map_err(|_| ()))
                // Get all notifications
                .and_then(|conn| {
                    notify::dsl::notify
                        .left_join(users::dsl::users)
                        .select((
                            notify::channel,
                            notify::type_,
                            notify::data,
                            users::access_token.nullable(),
                        ))
                        .load::<(i64, i32, Option<i64>, Option<String>)>(&*conn)
                        .map(|res: Vec<(i64, i32, Option<i64>, Option<String>)>| {
                            res.into_iter()
                                .map(|(channel, type_, data, access_token)| {
                                    (
                                        channel as u64,
                                        type_ as u8,
                                        data.map(|i| i as u64),
                                        access_token,
                                    )
                                })
                                .collect()
                        })
                        .map_err(|_| ())
                })
                // (channel_id, notification_type, discord_user_id, access_token)
                .and_then(|d: Vec<(u64, u8, Option<u64>, Option<String>)>| {
                    for notification in d {
                        sync(
                            Arc::clone(&data),
                            notification.0,
                            notification.1,
                            notification.2.unwrap(),
                            &notification.3.unwrap(),
                        );
                    }
                    Ok(())
                })
                .ok();

            thread::sleep(Duration::from_secs(60000))
        }
    });
}
