use serenity::{
    framework::{standard::Args, standard::Command, standard::CommandError},
    model::channel::Message,
    prelude::Context,
};
use crate::{
    wrappers::{Sqlite},
    schema::users::dsl::users,
    schema::users::dsl::subscribed,
    schema::users::dsl::discord_id
};
use diesel::{prelude::*, query_dsl::RunQueryDsl};

pub struct Subscribe;

impl Command for Subscribe {
    fn execute(&self, ctx: &mut Context, msg: &Message, _args: Args) -> Result<(), CommandError> {
        ctx.data
            .read()
            .get::<Sqlite>()
            .ok_or("Couldn't extract connection".to_owned())
            .and_then(|conn| conn.lock().map_err(|e| e.to_string()))
            .and_then(|conn| {
                let target = users.filter(discord_id.eq(msg.author.id.0 as i64));
                diesel::update(target).set(subscribed.eq(true)).execute(&*conn)
                    .map_err(|e| e.to_string())
            }).and_then(|_| Ok(()))
            .map_err(|e| {
                println!("{}", e);
                msg.author
                    .direct_message(&ctx, |m| {
                        m.embed(|embed| {
                            embed
                                .title("Error")
                                .description("There was an error subscribing you to the list!")
                                .field("Info", &e, true)
                                .color((237u8, 28u8, 36u8))
                        })
                    })
                    .ok();
                e.into()
            })
    }
}
