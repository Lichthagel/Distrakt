use crate::{
    messages::{full_movie, full_show},
    models::{User, Watchlist},
    wrappers::{Database, Trakt},
};
use chrono::Utc;
use rand::{seq::IteratorRandom, thread_rng};
use serenity::{
    client::Context,
    framework::standard::{Args, Command, CommandError},
    model::channel::Message,
};
use sled::Tree;
use std::sync::Arc;
use std::{cmp::min, string::ToString};
use time::Duration;
use trakt::models::{FullListItem, ListItemType};
use trakt::TraktApi;

fn get_watchlist(
    watchlists: Arc<Tree>,
    api: &TraktApi,
    user: &User,
) -> Result<Watchlist, String> {
    let watchlist = watchlists.get(&user.slug).map_err(|e| e.to_string())?;

    if let Some(watchlist) = watchlist {
        let watchlist: Watchlist = serde_cbor::from_slice(&watchlist).map_err(|e| e.to_string())?;

        if watchlist.last_downsync.signed_duration_since(Utc::now()) < Duration::hours(3) {
            return Ok(watchlist);
        }
    }

    Ok(Watchlist {
        last_downsync: Utc::now(),
        list: api.sync_watchlist_full(None, &user.access_token).map_err(|e| e.to_string())?,
    })
}

pub struct WatchlistList;

impl WatchlistList {
    fn run(ctx: &mut Context, msg: &Message) -> Result<(), String> {
        let lock = ctx.data.read();
        let users = lock
            .get::<Database>()
            .ok_or_else(|| "Couldn't extract users".to_owned())?
            .open_tree("users")
            .map_err(|e| e.to_string())?;

        let user = users
            .get(msg.author.id.0.to_le_bytes())
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "You are not logged in".to_owned())?;

        drop(lock);

        let user: User = serde_cbor::from_slice(&user).map_err(|e| e.to_string())?;

        let lock = ctx.data.read();

        let watchlists = lock
            .get::<Database>()
            .ok_or_else(|| "Couldn't extract users".to_owned())?
            .open_tree("watchlists")
            .map_err(|e| e.to_string())?;

        let watchlist = get_watchlist(watchlists, lock.get::<Trakt>().ok_or_else(|| "Couldn't extract API".to_owned())?, &user)?;

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

        let name = user
            .name
            .to_owned()
            .unwrap_or_else(|| user.username.to_owned());
        let username = user.username;
        let avatar = user.avatar.unwrap_or_default();

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
                                foot.text("Get refreshed when older than 3 hours")
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
        let users = lock
            .get::<Database>()
            .ok_or_else(|| "Couldn't extract users".to_owned())?
            .open_tree("users")
            .map_err(|e| e.to_string())?;

        let user = users
            .get(msg.author.id.0.to_le_bytes())
            .map_err(|e| e.to_string())?;

        drop(lock);

        if let Some(inner) = user {
            let user: User = serde_cbor::from_slice(&inner).map_err(|e| e.to_string())?;

            let lock = ctx.data.read();

            let api = lock
                .get::<Trakt>()
                .ok_or_else(|| "Couldn't extract api".to_owned())?;

            let watchlist: Vec<FullListItem> = api
                .sync_watchlist_full(None, &user.access_token)
                .map_err(|e| e.to_string())?;

            drop(lock);

            let item = watchlist
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

            let name = user
                .name
                .to_owned()
                .unwrap_or_else(|| user.username.to_owned());
            let username = user.username;
            let avatar = user.avatar.unwrap_or_default();

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
        } else {
            Err("You are not loggen in".to_owned())
        }
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
