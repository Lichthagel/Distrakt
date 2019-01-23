#[macro_use]
extern crate serde_derive;

mod commands;
mod config;
mod distrakt_trakt;

use crate::{
    commands::{auth::Login, owner::Shutdown},
    config::DistraktConfig,
    distrakt_trakt::Trakt,
};
use serenity::{
    framework::StandardFramework,
    model::prelude::{channel::Message, gateway::Game, gateway::Ready, id::UserId},
    prelude::{Context, EventHandler},
    Client,
};

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

    client.start().expect("couldn't start bot");
}
