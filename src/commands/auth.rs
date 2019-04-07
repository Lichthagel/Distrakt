use crate::{
    models::User,
    wrappers::{Trakt, Users},
};
use chrono::{offset::TimeZone, Utc};
use serenity::{
    framework::standard::{Args, Command, CommandError},
    model::channel::Message,
    prelude::Context,
};
use std::{thread, time::Duration as SleepDuration};
use time::Duration;
use trakt::error::Error;
use trakt::models::AuthenticationTokenResponse;

pub struct Login;

impl Login {
    fn run(ctx: &mut Context, msg: &Message) -> Result<(), String> {
        {
            let lock = ctx.data.read();
            let users = lock
                .get::<Users>()
                .ok_or_else(|| "Couldn't extract users".to_owned())?;

            if users
                .contains_key(msg.author.id.0.to_le_bytes())
                .map_err(|e| e.to_string())?
            {
                return Err("You are already logged in".to_owned());
            }
        }

        let code;
        {
            let lock = ctx.data.read();
            let api = lock
                .get::<Trakt>()
                .ok_or_else(|| "Couldn't extract api".to_owned())?;

            code = api.devices_authenticate().map_err(|e| e.to_string())?
        }

        let mut tokens = None;
        {
            let poll_until = Utc::now() + Duration::seconds((code.expires_in as i64) - 5);

            msg.author
                .direct_message(&ctx, |m| {
                    m.embed(|e| {
                        e.title("Login")
                            .url(&code.verification_url)
                            .description(format!(
                                "Go to {} and enter this code",
                                &code.verification_url
                            ))
                            .field("Code", &code.user_code, true)
                            .field("Expires at", poll_until.format("%H:%M:%S UTC"), true)
                            .timestamp(&poll_until)
                            .color((237u8, 28u8, 36u8))
                    })
                })
                .map_err(|e| e.to_string())?;

            'poll: while {
                thread::sleep(SleepDuration::from_secs(code.interval));

                let lock = ctx.data.read();
                let api = lock
                    .get::<Trakt>()
                    .ok_or_else(|| "Couldn't extract api".to_owned())?;

                let res = api.get_token(&code.device_code);

                if let Some(body) = match res {
                    Ok(body) => Ok(Some(body)),
                    Err(e) => {
                        if let Error::Response(res) = e {
                            if res.status().as_u16() != 400 {
                                Err(res.status().to_string())
                            } else {
                                Ok(None)
                            }
                        } else {
                            Err(e.to_string())
                        }
                    }
                }? {
                    tokens = Some(body);
                    break 'poll;
                };

                Utc::now() < poll_until
            } {}
        }

        drop(code);
        let tokens: AuthenticationTokenResponse =
            tokens.ok_or_else(|| "Couldn't log you in".to_owned())?;

        let settings;
        {
            let lock = ctx.data.read();
            let api = lock
                .get::<Trakt>()
                .ok_or_else(|| "Couldn't extract api".to_owned())?;

            settings = api
                .user_settings(&tokens.access_token)
                .map_err(|e| e.to_string())?;
        }

        {
            let lock = ctx.data.read();
            let users = lock
                .get::<Users>()
                .ok_or_else(|| "Couldn't extract users".to_owned())?;

            users
                .set(
                    msg.author.id.0.to_le_bytes(),
                    bincode::serialize(&User {
                        discord_id: msg.author.id.0,
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                        expires: Utc.timestamp(tokens.created_at as i64, 0).naive_utc()
                            + Duration::seconds(i64::from(tokens.expires_in)),
                        slug: settings.user.ids.slug.unwrap(),
                        username: settings.user.username,
                        name: settings.user.name,
                        private: settings.user.private,
                        vip: settings.user.vip,
                        cover_image: settings.account.cover_image,
                        avatar: settings.user.images.map(|i| i.avatar.full),
                        joined_at: settings.user.joined_at.map(|d| d.naive_utc()),
                    })
                    .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

impl Command for Login {
    fn execute(&self, ctx: &mut Context, msg: &Message, _args: Args) -> Result<(), CommandError> {
        match Login::run(ctx, msg) {
            Ok(()) => msg
                .author
                .direct_message(&ctx, |m| {
                    m.embed(|e| {
                        e.title("Success")
                            .description("You are now logged in. Have fun!")
                            .color((237u8, 28u8, 36u8))
                    })
                })
                .map(|_| ())
                .map_err(|e| e.into()),
            Err(e) => {
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
                Err(e.into())
            }
        }
    }
}
