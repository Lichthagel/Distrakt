#[macro_use]
extern crate serde_derive;

mod commands;
mod config;
mod messages;
mod models;
mod wrappers;

use crate::commands::watchlist::{WatchlistList, WatchlistRandom};
use crate::{
    commands::{auth::Login, owner::Shutdown},
    config::DistraktConfig,
    wrappers::{Trakt, Database},
};
use serenity::{
    framework::StandardFramework,
    model::prelude::{
        gateway::{Activity, Ready},
        id::UserId,
        Message,
    },
    prelude::{Context, EventHandler},
    Client,
};
use sled::Db;

struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "ping" {
            msg.channel_id.say(&ctx.http, "Pong!").unwrap();
        }
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::listening(&format!(
            "{}login",
            ctx.data.read().get::<DistraktConfig>().unwrap().prefix
        )));
        println!("connected to {} guilds", ready.guilds.len());
    }
}

fn main() {
    let conf = DistraktConfig::load();

    let mut client =
        Client::new(conf.discord_token.as_str(), Handler).expect("error creating client");

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix(conf.prefix.as_str())
                    .owners(conf.owners.iter().map(|i| UserId(*i)).collect())
            })
            .command("shutdown", |c| {
                c.owners_only(true)
                    .cmd(Shutdown)
                    .before(|ctx, msg| msg.reply(ctx, "shutting down").is_ok())
            })
            .command("login", |c| c.cmd(Login))
            .group("watchlist", |g| {
                g.prefix("watchlist")
                    .default_cmd(WatchlistList)
                    .command("random", |c| c.cmd(WatchlistRandom))
            }),
    );

    {
        let api = Trakt::new(conf.trakt_id.clone(), Some(conf.trakt_secret.clone()));

        let mut data = client.data.write();

        data.insert::<Trakt>(api);
    }

    {
        let db = Db::start_default("db").unwrap();

        client.data.write().insert::<Database>(Database::new(db));
    }

    client.data.write().insert::<DistraktConfig>(conf);

    client.start().expect("couldn't start bot");
}
