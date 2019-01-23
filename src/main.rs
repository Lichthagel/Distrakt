#[macro_use]
extern crate serde_derive;

mod commands;
mod config;

use crate::{commands::owner::Shutdown, config::DistraktConfig};
use serenity::{
    client::{Context, EventHandler},
    framework::StandardFramework,
    model::{channel::Message, gateway::Ready, id::UserId},
    Client,
};

struct Handler;

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {
        if msg.content == "ping" {
            msg.channel_id.say("Pong!").unwrap();
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
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
                c.prefix("+")
                    .owners(conf.owners.into_iter().map(|i| UserId(i)).collect())
            })
            .command("shutdown", |c| {
                c.owners_only(true)
                    .cmd(Shutdown)
                    .before(|_, msg| msg.reply("shutting down").is_ok())
            }),
    );

    client.start().expect("couldn't start bot");
}
