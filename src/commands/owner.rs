use crate::wrappers::{Database, Trakt};
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

        if let Some(users) = lock.get::<Database>() {
            users.flush()?;
        };

        lock.remove::<Database>();
        lock.remove::<Trakt>();

        ctx.shard.shutdown_clean();

        thread::sleep(Duration::from_secs(1));

        process::exit(0);
    }
}

pub struct Db;

impl Command for Db {
    fn execute(&self, ctx: &mut Context, _msg: &Message, _args: Args) -> Result<(), CommandError> {
        let lock = ctx.data.read();
        let db = lock
            .get::<Database>()
            .ok_or_else(|| "Couldn't extract DB".to_owned())?;

        db.drop_tree(b"watchlists").map_err(|e| e.to_string())?;

        Ok(())
    }
}
