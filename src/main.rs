#[macro_use]
extern crate serde_derive;

mod config;

use crate::config::DistraktConfig;
use serenity::{
    client::{Context, EventHandler},
    model::{channel::Message, gateway::Ready},
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

    client.start().expect("couldn't start bot");
}
