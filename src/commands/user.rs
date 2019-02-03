use crate::wrappers::Trakt;
use crate::{schema::users::dsl::*, wrappers::Sqlite};
use diesel::prelude::*;
use serenity::{
    framework::standard::{Args, Command, CommandError},
    model::prelude::Message,
    prelude::Context,
};

pub struct WhoAmI;

impl Command for WhoAmI {
    fn execute(&self, ctx: &mut Context, msg: &Message, _: Args) -> Result<(), CommandError> {
        ctx.data
            .read()
            .get::<Sqlite>()
            .ok_or("Couldn't extract SQL connection".to_owned())
            .and_then(|conn| conn.lock().map_err(|e| e.to_string()))
            .and_then(|conn| {
                users
                    .filter(discord_id.eq(msg.author.id.0 as i64))
                    .limit(1)
                    .select(access_token)
                    .load::<String>(&*conn)
                    .map_err(|e| e.to_string())
            })
            .and_then(|tokens: Vec<String>| match tokens.get(0) {
                Some(token) => ctx
                    .data
                    .read()
                    .get::<Trakt>()
                    .ok_or("Couldn't extract API".to_owned())
                    .and_then(|api| api.user_settings(token).map_err(|e| e.to_string())),
                None => Err("You are not logged in".to_owned()),
            })
            .and_then(|settings| {
                msg.reply(
                    &ctx,
                    format!(
                        "{} ({})",
                        &settings.user.username,
                        &settings.user.name.unwrap_or(settings.user.username.clone())
                    )
                    .as_str(),
                )
                .map_err(|e| e.to_string())
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
