use serenity::{
    framework::{standard::Args, standard::Command, standard::CommandError},
    model::channel::Message,
    prelude::Context,
};
use std::{
    process,
    thread,
    time::Duration,
    sync::Mutex
};
use crate::wrappers::Wrapper;
use trakt::TraktApi;
use postgres::Connection;

pub struct Shutdown;

impl Command for Shutdown {
    fn execute(&self, ctx: &mut Context, _msg: &Message, _args: Args) -> Result<(), CommandError> {
        thread::sleep(Duration::from_secs(1));

        let mut lock = ctx.data.write();

        if let Some(cache) = lock.get::<Wrapper<sled::Db>>() {
            cache.flush()?;
        };

        lock.remove::<Wrapper<ClearCache>>();
        lock.remove::<Wrapper<TraktApi>>();
        lock.remove::<Wrapper<Mutex<Connection>>>();

        ctx.shard.shutdown_clean();

        thread::sleep(Duration::from_secs(1));

        process::exit(0);
    }
}

pub struct ClearCache;

impl Command for ClearCache {
    fn execute(&self, ctx: &mut Context, _msg: &Message, _args: Args) -> Result<(), CommandError> {
        let lock = ctx.data.read();
        let db = lock
            .get::<Wrapper<sled::Db>>()
            .ok_or_else(|| "Couldn't extract DB".to_owned())?;

        db.clear()?;

        Ok(())
    }
}
