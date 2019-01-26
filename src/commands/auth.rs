use crate::{
    models::User,
    schema::users::discord_id,
    schema::users::dsl::users,
    schema::users::table,
    wrappers::{Sqlite, Trakt},
};
use chrono::{offset::TimeZone, Utc};
use diesel::{prelude::*, query_dsl::RunQueryDsl};
use serenity::{
    framework::standard::{Args, Command, CommandError},
    model::channel::Message,
    prelude::Context,
};
use std::{thread, time::Duration as SleepDuration};
use time::Duration;
use trakt::error::Error;

pub struct Login;

impl Command for Login {
    fn execute(&self, ctx: &mut Context, msg: &Message, _args: Args) -> Result<(), CommandError> {
        ctx.data
            .read()
            .get::<Sqlite>()
            .ok_or("Couldn't extract connection".to_owned())
            .and_then(|conn| conn.lock().map_err(|e| e.to_string()))
            .and_then(|conn| {
                users
                    .filter(discord_id.eq(msg.author.id.0 as i64))
                    .count()
                    .get_result(&*conn)
                    .map_err(|e| e.to_string())
            })
            .and_then(|count: i64| {
                if count == 0 {
                    Ok(())
                } else {
                    Err("You are already logged in".to_owned())
                }
            })
            .and_then(|_| {
                ctx.data
                    .read()
                    .get::<Trakt>()
                    .ok_or("Couldn't extract api".to_owned())
                    .and_then(|api| api.devices_authenticate().map_err(|e| e.to_string()))
            })
            .and_then(|code| {
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
                    .map_err(|e| e.to_string())
                    .map(|_| (code, poll_until))
            })
            .and_then(|(code, poll_until)| {
                ctx.data
                    .read()
                    .get::<Trakt>()
                    .ok_or("Couldn't extract api".to_owned())
                    .and_then(|api| {
                        let mut tokens = None;

                        'poll: while {
                            thread::sleep(SleepDuration::from_secs(code.interval));

                            let res = api.get_token(&code.device_code);

                            match match res {
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
                                Some(body) => {
                                    tokens = Some(body);
                                    break 'poll;
                                }
                                None => {}
                            };

                            Utc::now() < poll_until
                        } {}

                        match tokens {
                            Some(tok) => Ok(tok),
                            None => Err("Couldn't log you in".to_owned()),
                        }
                    })
            })
            .and_then(|tokens| {
                ctx.data
                    .read()
                    .get::<Sqlite>()
                    .ok_or("Couldn't get SQL connection")?
                    .lock()
                    .map_err(|e| e.to_string())
                    .and_then(|conn| {
                        let user = User {
                            discord_id: msg.author.id.0 as i64,
                            access_token: tokens.access_token,
                            refresh_token: tokens.refresh_token,
                            expires: Utc.timestamp(tokens.created_at as i64, 0).naive_utc()
                                + Duration::seconds(tokens.expires_in as i64),
                            subscribed: false
                        };

                        diesel::insert_into(table)
                            .values(&user)
                            .execute(&*conn)
                            .map_err(|e| e.to_string())
                    })
            })
            .and_then(|_| {
                msg.author
                    .direct_message(&ctx, |m| {
                        m.embed(|e| {
                            e.title("Success")
                                .description("You are now logged in. Have fun!")
                                .color((237u8, 28u8, 36u8))
                        })
                    })
                    .map_err(|e| e.to_string())
            })
            .and_then(|_| Ok(()))
            .map_err(|e| {
                println!("{}", e);
                msg.author
                    .direct_message(&ctx, |m| {
                        m.embed(|embed| {
                            embed
                                .title("Error")
                                .description("There was an error logging you in")
                                .field("Info", &e, true)
                                .color((237u8, 28u8, 36u8))
                        })
                    })
                    .ok();
                e.into()
            })
    }
}
