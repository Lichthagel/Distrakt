use crate::wrappers::{Trakt, Users};
use serenity::{
    framework::{standard::Args, standard::Command, standard::CommandError},
    model::channel::Message,
    prelude::Context,
};
use std::{process, thread, time::Duration};

pub struct Shutdown;

impl Command for Shutdown {
    fn execute(&self, ctx: &mut Context, _msg: &Message, _args: Args) -> Result<(), CommandError> {
        thread::sleep(Duration::from_secs(1));

        let mut lock = ctx.data.write();

        if let Some(users) = lock.get::<Users>() {
            users.flush()?;
        };

        lock.remove::<Users>();
        lock.remove::<Trakt>();

        ctx.shard.shutdown_clean();

        thread::sleep(Duration::from_secs(1));

        process::exit(0);
    }
}
