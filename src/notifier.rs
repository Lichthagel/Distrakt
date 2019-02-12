use crate::{
    models::{Episode, Movie, Notification, Show},
    schema::{episodes, movies, notifications, notify, shows, users},
    wrappers::{Sqlite, Trakt},
};
use chrono::{offset::TimeZone, DateTime, NaiveDateTime, Utc};
use diesel::{
    prelude::{QueryResult, SqliteConnection},
    query_dsl::{QueryDsl, RunQueryDsl},
    ExpressionMethods, Insertable, NullableExpressionMethods,
};
use serenity::{http::raw::Http, model::prelude::id::ChannelId, prelude::RwLock};
use std::{sync::Arc, thread, time::Duration};
use time::Duration as TimeDuration;
use trakt::{
    extended_info::ExtendedInfoFull,
    models::{
        FullCalendarMovie as TraktCalendarMovie, FullEpisode as TraktEpisode, FullShow as TraktShow,
    },
};
use typemap::ShareMap;

fn insert_show(conn: &SqliteConnection, show: TraktShow) -> QueryResult<usize> {
    Show::from(show).insert_into(shows::table).execute(conn)
}

fn insert_episode(
    conn: &SqliteConnection,
    episode: TraktEpisode,
    first_aired: DateTime<Utc>,
    show_slug: String,
) -> QueryResult<usize> {
    Episode::from((episode, first_aired, show_slug))
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
    channel: u64,
    type_: u8,
    _discord_id: u64,
    access_token: &str,
) {
    if type_ % 2 == 0 {
        data.read().get::<Trakt>().map(|api| {
            // movies
            if type_ & 4 == 4 {
                api.calendar_my_movies(access_token)
                    .start_date(Utc::today())
                    .days(4)
                    .full()
                    .execute()
                    .map(|res| {
                        for movie in res {
                            let id = movie.movie.ids.trakt.unwrap().clone();
                            data.read()
                                .get::<Sqlite>()
                                .ok_or(())
                                .and_then(|sql| sql.lock().map_err(|_| ()))
                                .and_then(|conn| {
                                    insert_movie(&*conn, movie)
                                        .and_then(|_| insert_notification(&*conn, channel, id))
                                        .map_err(|_| ())
                                })
                                .ok();
                        }
                    })
                    .ok();
            }
            // shows
            if type_ & 8 == 8 {
                api.calendar_my_shows(access_token)
                    .start_date(Utc::today())
                    .days(4)
                    .full()
                    .execute()
                    .map(|res| {
                        for show in res {
                            data.read()
                                .get::<Sqlite>()
                                .ok_or(())
                                .and_then(|conn| conn.lock().map_err(|_| ()))
                                .and_then(|conn| {
                                    let id = show.episode.ids.trakt.unwrap().clone();
                                    let show_slug = show.show.ids.slug.as_ref().unwrap().clone();

                                    insert_show(&*conn, show.show).ok();
                                    insert_episode(
                                        &*conn,
                                        show.episode,
                                        show.first_aired,
                                        show_slug,
                                    )
                                    .ok();
                                    insert_notification(&*conn, channel, id).ok();
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
            println!("syncing");
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

pub fn notify_thread(data: Arc<RwLock<ShareMap>>, http: Arc<Http>) {
    thread::spawn(move || loop {
        println!("notifying");
        data.read()
            .get::<Sqlite>()
            .ok_or(())
            .and_then(|sql| sql.lock().map_err(|_| ()))
            .and_then(|conn| {
                episodes::table
                    .filter(
                        episodes::first_aired.lt(Utc::now().naive_utc() + TimeDuration::minutes(5)),
                    )
                    .filter(episodes::first_aired.gt(Utc::now().naive_utc()))
                    .inner_join(shows::dsl::shows)
                    .select((
                        episodes::trakt_id,
                        shows::title,
                        episodes::season_num,
                        episodes::episode_num,
                        episodes::title,
                        episodes::first_aired,
                        episodes::overview,
                        episodes::runtime,
                    ))
                    .load::<(
                        i64,
                        String,
                        i32,
                        i32,
                        String,
                        Option<NaiveDateTime>,
                        Option<String>,
                        Option<i32>,
                    )>(&*conn)
                    .map_err(|_| ())
            })
            .map(
                |res: Vec<(
                    i64,
                    String,
                    i32,
                    i32,
                    String,
                    Option<NaiveDateTime>,
                    Option<String>,
                    Option<i32>,
                )>| {
                    res.iter()
                        .map(
                            |(
                                trakt_id,
                                show_title,
                                season_num,
                                episode_num,
                                episode_title,
                                first_aired,
                                overview,
                                runtime,
                            )| {
                                (
                                    *trakt_id as u64,
                                    show_title.to_owned(),
                                    *season_num,
                                    *episode_num,
                                    episode_title.to_owned(),
                                    first_aired
                                        .map(|first_aired| Utc.from_utc_datetime(&first_aired)),
                                    overview.to_owned(),
                                    runtime.unwrap() as u32,
                                )
                            },
                        )
                        .collect()
                },
            )
            .and_then(
                |res: Vec<(
                    u64,
                    String,
                    i32,
                    i32,
                    String,
                    Option<DateTime<Utc>>,
                    Option<String>,
                    u32,
                )>| {
                    for (
                        trakt_id,
                        show_title,
                        season_num,
                        episode_num,
                        episode_title,
                        first_aired,
                        overview,
                        runtime,
                    ) in res
                    {
                        data.read()
                            .get::<Sqlite>()
                            .ok_or(())
                            .and_then(|sql| sql.lock().map_err(|_| ()))
                            .and_then(|conn| {
                                notifications::table
                                    .select(notifications::channel)
                                    .filter(notifications::trakt_id.eq(trakt_id as i64))
                                    .load::<i64>(&*conn)
                                    .map(|res| res.iter().map(|i| ChannelId(*i as u64)).collect())
                                    .and_then(|res: Vec<ChannelId>| {
                                        for channel in res {
                                            channel
                                                .send_message(&http, |m| {
                                                    m.embed(|mut e| {
                                                        if let Some(overview) = &overview {
                                                            e = e
                                                                .field("Overview", overview, false);
                                                        }

                                                        e.title("New Episode Notification")
                                                            .url(format!(
                                                                "https://trakt.tv/episodes/{}",
                                                                trakt_id
                                                            ))
                                                            .description(format!(
                                                                "**{}**\nEpisode {}x{} \"{}\"",
                                                                show_title,
                                                                season_num,
                                                                episode_num,
                                                                episode_title
                                                            ))
                                                            .field(
                                                                "Runtime",
                                                                format!("{} minutes", runtime),
                                                                true,
                                                            )
                                                            .timestamp(
                                                                first_aired.unwrap().to_rfc3339(),
                                                            )
                                                    })
                                                })
                                                .ok();
                                        }

                                        Ok(())
                                    })
                                    .and_then(|_| {
                                        diesel::delete(
                                            notifications::table.filter(
                                                notifications::trakt_id.eq(trakt_id as i64),
                                            ),
                                        )
                                        .execute(&*conn)
                                    })
                                    .map_err(|_| ())
                            })
                            .ok();
                    }
                    Ok(())
                },
            )
            .and(
                data.read()
                    .get::<Sqlite>()
                    .ok_or(())
                    .and_then(|sql| sql.lock().map_err(|_| ()))
                    .and_then(|conn| {
                        movies::table
                            .filter(movies::released.eq(Utc::today().naive_utc()))
                            .load::<Movie>(&*conn)
                            .map_err(|_| ())
                    })
                    .and_then(|res: Vec<Movie>| {
                        for movie in res {
                            data.read()
                                .get::<Sqlite>()
                                .ok_or(())
                                .and_then(|sql| sql.lock().map_err(|_| ()))
                                .and_then(|conn| {
                                    notifications::table
                                        .select(notifications::channel)
                                        .filter(notifications::trakt_id.eq(movie.trakt_id))
                                        .load::<i64>(&*conn)
                                        .map(|res| {
                                            res.iter().map(|i| ChannelId(*i as u64)).collect()
                                        })
                                        .and_then(|res: Vec<ChannelId>| {
                                            for channel in res {
                                                channel
                                                    .send_message(&http, |m| {
                                                        m.embed(|mut e| {
                                                            let movie = &movie;
                                                            if let Some(overview) = &movie.overview {
                                                                e = e.field(
                                                                    "Overview", overview, false,
                                                                );
                                                            }

                                                            if let Some(trailer) = &movie.trailer {
                                                                e = e.field(
                                                                    "Trailer", trailer, true,
                                                                );
                                                            }

                                                            if let Some(homepage) = &movie.homepage {
                                                                e = e.field(
                                                                    "Homepage", homepage, true,
                                                                );
                                                            }

                                                            e.title("New Movie Notification")
                                                                .url(format!(
                                                                    "https://trakt.tv/movies/{}",
                                                                    movie.slug
                                                                ))
                                                                .description(format!(
                                                                    "Movie \"{}\" aired",
                                                                    movie.title
                                                                ))
                                                                .field(
                                                                    "Runtime",
                                                                    format!(
                                                                        "{} minutes",
                                                                        movie.runtime.unwrap()
                                                                    ),
                                                                    true,
                                                                )
                                                                .timestamp(
                                                                    Utc.from_utc_date(
                                                                        &movie.released.unwrap(),
                                                                    )
                                                                    .and_hms(0, 0, 0)
                                                                    .to_rfc3339(),
                                                                )
                                                        })
                                                    })
                                                    .ok();
                                            }

                                            Ok(())
                                        })
                                        .and_then(|_| {
                                            diesel::delete(
                                                notifications::table.filter(
                                                    notifications::trakt_id.eq(movie.trakt_id),
                                                ),
                                            )
                                            .execute(&*conn)
                                        })
                                        .map_err(|_| ())
                                })
                                .ok();
                        }

                        Ok(())
                    }),
            )
            .ok();
        thread::sleep(Duration::from_secs(300));
    });
}
