use crate::{
    wrappers::Wrapper,
};
use chrono::{offset::TimeZone, Utc};
use serenity::{
    framework::standard::{Args, Command, CommandError},
    model::channel::Message,
    prelude::Context,
};
use std::{thread, time::Duration as SleepDuration};
use time::Duration;
use trakt::{error::Error, models::AuthenticationTokenResponse, TraktApi};
use postgres::Connection;
use std::sync::Mutex;

pub struct Login;

impl Login {
    fn run(ctx: &mut Context, msg: &Message) -> Result<(), String> {
        {
            let lock = ctx.data.read();
            let db = lock.get::<Wrapper<Mutex<Connection>>>().ok_or_else(|| "Couldn't get database")?;
            let db = db.lock().map_err(|e| e.to_string())?;

            let res = db.query("SELECT * FROM users WHERE discord_id = $1", &[&(msg.author.id.0 as i64)]).map_err(|e| e.to_string())?;

            if res.len() > 0 {
                return Err("You are already logged in".to_owned());
            }
        }

        let code;
        {
            let lock = ctx.data.read();
            let api = lock
                .get::<Wrapper<TraktApi>>()
                .ok_or_else(|| "Couldn't extract api".to_owned())?;

            code = api.oauth_device_code().map_err(|e| e.to_string())?
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
                    .get::<Wrapper<TraktApi>>()
                    .ok_or_else(|| "Couldn't extract api".to_owned())?;

                let res = api.oauth_device_token(&code.device_code);

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
                .get::<Wrapper<TraktApi>>()
                .ok_or_else(|| "Couldn't extract api".to_owned())?;

            settings = api
                .user_settings(&tokens.access_token)
                .map_err(|e| e.to_string())?;
        }

        {
            let lock = ctx.data.read();
            let db = lock.get::<Wrapper<Mutex<Connection>>>().ok_or_else(|| "Couldn't get database")?;
            let db = db.lock().map_err(|e| e.to_string())?;

            db.execute("INSERT INTO users (discord_id, access_token, refresh_token, expires, slug, username, name, private, vip, cover_image, avatar, joined_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12);", &[
                &(msg.author.id.0 as i64),
                &tokens.access_token,
                &tokens.refresh_token,
                &(Utc.timestamp(tokens.created_at as i64, 0).naive_utc() + Duration::seconds(tokens.expires_in as i64)),
                &settings.user.ids.slug.unwrap(),
                &settings.user.username,
                &settings.user.name,
                &settings.user.private,
                &settings.user.vip,
                &settings.account.cover_image,
                &settings.user.images.map(|i| i.avatar.full),
                &settings.user.joined_at.map(|d| d.naive_utc())
            ]).map_err(|e| e.to_string())?;
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
                .map_err(Into::into),
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
