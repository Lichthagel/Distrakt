use crate::{schema::notify::dsl::*, wrappers::Sqlite};
use diesel::{prelude::*, query_dsl::RunQueryDsl};
use serenity::{
    framework::{standard::Args, standard::Command, standard::CommandError},
    model::channel::Message,
    prelude::Context,
};

pub struct Notify;

impl Command for Notify {
    fn execute(&self, ctx: &mut Context, msg: &Message, _args: Args) -> Result<(), CommandError> {
        // TODO check if user has channel permission (private channel, or admin on guild)
        // TODO setup different notifications
        ctx.data
            .read()
            .get::<Sqlite>()
            .ok_or("Couldn't extract connection".to_owned())
            .and_then(|conn| conn.lock().map_err(|e| e.to_string()))
            .and_then(|conn| {
                diesel::insert_into(notify)
                    .values((
                        channel.eq(msg.channel_id.0 as i64),
                        data.eq(msg.author.id.0 as i64),
                    ))
                    .execute(&*conn)
                    .map_err(|e| e.to_string())
            })
            .and_then(|res| {
                if res > 0 {
                    msg.author
                        .direct_message(&ctx, |m| {
                            m.embed(|e| {
                                e.title("Success")
                                    .description("You have successfully subscribed!")
                                    .color((237u8, 28u8, 36u8))
                            })
                        })
                        .map_err(|e| e.to_string())
                } else {
                    Err("You are not signed in".to_owned())
                }
            })
            .and_then(|_| Ok(()))
            .map_err(|e| {
                msg.author
                    .direct_message(&ctx, |m| {
                        m.embed(|embed| {
                            embed
                                .title("Error")
                                .description(&e)
                                .color((237u8, 28u8, 36u8))
                        })
                    })
                    .ok();
                e.into()
            })
    }
}
