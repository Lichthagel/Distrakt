use crate::{
    messages::{full_movie, full_show},
    models::{Watchlist},
    wrappers::Wrapper,
};
use chrono::Utc;
use rand::{seq::IteratorRandom, thread_rng};
use serenity::{
    client::Context,
    framework::standard::{Args, Command, CommandError},
    model::channel::Message,
};
use sled::{Tree, Db};
use std::{
    sync::{
        Arc,
        Mutex,
    },
    cmp::min,
    string::ToString,
};
use time::Duration;
use trakt::{
    models::ListItemType,
    TraktApi,
};
use postgres::Connection;

fn get_watchlist(
    watchlists: Arc<Tree>,
    api: &TraktApi,
    slug: &str,
    access_token: &str,
) -> Result<Watchlist, String> {
    let watchlist = watchlists.get(slug).map_err(|e| e.to_string())?;

    if let Some(watchlist) = watchlist {
        let watchlist: Watchlist = serde_cbor::from_slice(&watchlist).map_err(|e| e.to_string())?;

        if watchlist.last_downsync.signed_duration_since(Utc::now()) < Duration::hours(3) {
            return Ok(watchlist);
        }
    }

    let watchlist = Watchlist {
        last_downsync: Utc::now(),
        list: api.sync_watchlist_full(None, access_token).map_err(|e| e.to_string())?,
    };

    watchlists.set(slug, serde_cbor::to_vec(&watchlist).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;

    Ok(watchlist)
}

pub struct WatchlistList;

impl WatchlistList {
    fn run(ctx: &mut Context, msg: &Message) -> Result<(), String> {
        let lock = ctx.data.read();
        let db = lock.get::<Wrapper<Mutex<Connection>>>().ok_or_else(|| "Couldn't get database")?;
        let db = db.lock().map_err(|e| e.to_string())?;

        let res = db.query("SELECT slug, access_token, name, username, avatar FROM users WHERE discord_id = $1", &[&(msg.author.id.0 as i64)]).map_err(|e| e.to_string())?;

        drop(db);
        drop(lock);

        if res.len() == 0 {
            return Err("You are not logged in".to_owned());
        }

        let res = res.get(0);

        let lock = ctx.data.read();

        let watchlists = lock
            .get::<Wrapper<Db>>()
            .ok_or_else(|| "Couldn't get watchlists".to_owned())?
            .open_tree("watchlists")
            .map_err(|e| e.to_string())?;

        let watchlist = get_watchlist(watchlists, lock.get::<Wrapper<TraktApi>>().ok_or_else(|| "Couldn't extract API".to_owned())?, &res.get::<&str, String>("slug"), &res.get::<&str, String>("access_token"))?;

        drop(lock);

        let time = &watchlist.last_downsync;

        let message = watchlist
            .list
            .into_iter()
            .map(|item| match item.item_type {
                ListItemType::Movie => format!(
                    ":movie_camera: [{}] {}",
                    item.rank,
                    item.movie.unwrap().title
                ),
                ListItemType::Show => {
                    format!(":film_frames: [{}] {}", item.rank, item.show.unwrap().title)
                }
                ListItemType::Season => format!(
                    ":film_frames: [{}] {} Season {}",
                    item.rank,
                    item.show.unwrap().title,
                    item.season.unwrap().number
                ),
                ListItemType::Episode => format!(
                    ":film_frames: [{}] {}",
                    item.rank.to_owned(),
                    item.episode
                        .as_ref()
                        .unwrap()
                        .title
                        .to_owned()
                        .unwrap_or_else(|| format!("Episode {}", item.episode.unwrap().number))
                ),
                ListItemType::Person => {
                    format!(":mens: [{}] {}", item.rank, item.person.unwrap().name)
                }
            })
            .collect::<Vec<String>>();

        let name = res.get::<&str, Option<String>>("name")
            .unwrap_or_else(|| res.get("username"));
        let username: String = res.get("username");
        let avatar = res.get::<&str, Option<String>>("avatar").unwrap_or_default();

        for i in 1..(message.len() / 20) + 2 {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|embed| {
                        embed
                            .title(":notepad_spiral: W A T C H L I S T")
                            .color((237u8, 28u8, 36u8))
                            .description(
                                message[((i - 1) * 20)..min(i * 20, message.len())].join("\n"),
                            )
                            .author(|author| {
                                author
                                    .name(&name)
                                    .url(&format!("https://trakt.tv/users/{}", &username))
                                    .icon_url(&avatar)
                            })
                            .url(&format!("https://trakt.tv/users/{}/watchlist", &username))
                            .timestamp(time)
                            .footer(|foot| {
                                foot.text("Gets refreshed when older than 3 hours")
                            })
                    })
                })
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

impl Command for WatchlistList {
    fn execute(&self, ctx: &mut Context, msg: &Message, _: Args) -> Result<(), CommandError> {
        let result = WatchlistList::run(ctx, msg);

        result.map_err(|e| {
            msg.author
                .direct_message(&ctx, |m| {
                    m.embed(|embed| {
                        embed
                            .title("Error")
                            .description(&e)
                            .color((237u8, 28u8, 36u8))
                    })
                })
                .ok();
            e.into()
        })
    }
}

pub struct WatchlistRandom;

impl WatchlistRandom {
    fn run(ctx: &mut Context, msg: &Message) -> Result<(), String> {
        let lock = ctx.data.read();
        let db = lock.get::<Wrapper<Mutex<Connection>>>().ok_or_else(|| "Couldn't get database")?;
        let db = db.lock().map_err(|e| e.to_string())?;

        let res = db.query("SELECT slug, access_token, name, username, avatar FROM users WHERE discord_id = $1", &[&(msg.author.id.0 as i64)]).map_err(|e| e.to_string())?;

        drop(db);
        drop(lock);

        if res.len() == 0 {
            return Err("You are not logged in".to_owned());
        }

        let res = res.get(0);

        let lock = ctx.data.read();

        let watchlists = lock.get::<Wrapper<Db>>().ok_or_else(|| "Couldn't get watchlists".to_owned())?.open_tree("watchlists").map_err(|e| e.to_string())?;

        let api = lock
            .get::<Wrapper<TraktApi>>()
            .ok_or_else(|| "Couldn't extract api".to_owned())?;

        let watchlist = get_watchlist(watchlists, api, &res.get::<&str, String>("slug"), &res.get::<&str, String>("access_token"))?;

        drop(lock);

        let item = watchlist
            .list
            .into_iter()
            .choose(&mut thread_rng())
            .ok_or_else(|| "Your watchlist is empty".to_owned())?;

        if item.item_type == ListItemType::Movie {
            msg.channel_id
                .send_message(&ctx.http, |m| full_movie(item.movie.as_ref().unwrap(), m))
                .map_err(|e| e.to_string())?;
            return Ok(());
        } else if item.item_type == ListItemType::Show {
            msg.channel_id
                .send_message(&ctx.http, |m| full_show(item.show.as_ref().unwrap(), m))
                .map_err(|e| e.to_string())?;
            return Ok(());
        }

        let message = match item.item_type {
            ListItemType::Movie => format!(
                ":movie_camera: [{}] {}",
                item.rank,
                item.movie.unwrap().title
            ),
            ListItemType::Show => {
                format!(":film_frames: [{}] {}", item.rank, item.show.unwrap().title)
            }
            ListItemType::Season => format!(
                ":film_frames: [{}] {} Season {}",
                item.rank,
                item.show.unwrap().title,
                item.season.unwrap().number
            ),
            ListItemType::Episode => format!(
                ":film_frames: [{}] {}",
                item.rank.to_owned(),
                item.episode
                    .as_ref()
                    .unwrap()
                    .title
                    .to_owned()
                    .unwrap_or_else(|| format!("Episode {}", item.episode.unwrap().number))
            ),
            ListItemType::Person => {
                format!(":mens: [{}] {}", item.rank, item.person.unwrap().name)
            }
        };

        let name = res.get::<&str, Option<String>>("name")
            .unwrap_or_else(|| res.get("username"));
        let username: String = res.get("username");
        let avatar = res.get::<&str, Option<String>>("avatar").unwrap_or_default();

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|embed| {
                    embed
                        .title(":notepad_spiral: W A T C H L I S T")
                        .color((237u8, 28u8, 36u8))
                        .description(message)
                        .author(|author| {
                            author
                                .name(&name)
                                .url(&format!("https://trakt.tv/users/{}", &username))
                                .icon_url(&avatar)
                        })
                        .url(&format!("https://trakt.tv/users/{}/watchlist", &username))
                })
            })
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

impl Command for WatchlistRandom {
    fn execute(&self, ctx: &mut Context, msg: &Message, _: Args) -> Result<(), CommandError> {
        let result = WatchlistRandom::run(ctx, msg);

        result.map_err(|e| {
            msg.author
                .direct_message(&ctx, |m| {
                    m.embed(|embed| {
                        embed
                            .title("Error")
                            .description(&e)
                            .color((237u8, 28u8, 36u8))
                    })
                })
                .ok();
            e.into()
        })
    }
}
