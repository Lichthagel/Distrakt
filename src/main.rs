#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

mod commands;
mod config;
mod models;
mod wrappers;
mod schema;

use crate::{
    commands::{auth::Login, owner::Shutdown},
    config::DistraktConfig,
    wrappers::{Sqlite, Trakt},
};
use diesel::prelude::*;
use serenity::{
    framework::StandardFramework,
    model::prelude::*,
    prelude::{Context, EventHandler},
    Client,
};
use std::env;

struct Handler;

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        if msg.content == "ping" {
            msg.channel_id.say("Pong!").unwrap();
        }
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_game(Game::listening("+login"));
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
                    .owners(conf.owners.iter().map(|i| UserId(i.clone())).collect())
            })
            .command("shutdown", |c| {
                c.owners_only(true)
                    .cmd(Shutdown)
                    .before(|_, msg| msg.reply("shutting down").is_ok())
            })
            .command("login", |c| c.cmd(Login)),
    );

    {
        let api = Trakt::new(conf.trakt_id, Some(conf.trakt_secret));

        let mut data = client.data.lock();

        data.insert::<Trakt>(api);
    }

    {
        let conn = SqliteConnection::establish(
            format!("{}/distrakt.db", env::current_dir().unwrap().display()).as_str(),
        )
        .expect("Couldn't connect to database");

        let mut data = client.data.lock();

        data.insert::<Sqlite>(Sqlite::new(conn));
    }

    client.start().expect("couldn't start bot");
}
