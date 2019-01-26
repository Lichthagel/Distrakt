#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

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
    fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "ping" {
            msg.channel_id.say(&ctx.http, "Pong!").unwrap();
        }
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::listening("+login"));
        println!("connected to {} guilds", ready.guilds.len());
    }
}

embed_migrations!();

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
                    .before(|ctx, msg| msg.reply(ctx, "shutting down").is_ok())
            })
            .command("login", |c| c.cmd(Login)),
    );

    {
        let api = Trakt::new(conf.trakt_id, Some(conf.trakt_secret));

        let mut data = client.data.write();

        data.insert::<Trakt>(api);
    }

    {
        let conn = SqliteConnection::establish(
            format!("{}/distrakt.db", env::current_dir().unwrap().display()).as_str(),
        )
        .expect("Couldn't connect to database");

        let _ = embedded_migrations::run_with_output(&conn, &mut std::io::stdout());

        let mut data = client.data.write();

        data.insert::<Sqlite>(Sqlite::new(conn));
    }

    client.start().expect("couldn't start bot");
}
