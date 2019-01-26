use crate::wrappers::Sqlite;
use crate::{models::User, schema::users, wrappers::Trakt};
use chrono::{offset::TimeZone, DateTime, Utc};
use serenity::{
    framework::standard::{Args, Command, CommandError},
    model::channel::Message,
    prelude::Context,
};
use std::{thread, time::Duration as SleepDuration};
use time::Duration;
use trakt::error::Error;
use diesel::query_dsl::RunQueryDsl;

pub struct Login;

impl Command for Login {
    fn execute(&self, ctx: &mut Context, msg: &Message, _args: Args) -> Result<(), CommandError> {
        let data = ctx.data.lock();

        // TODO use futures

        let api = data
            .get::<Trakt>()
            .ok_or(CommandError("Couldn't extract api".to_owned()))?;

        // TODO check if already logged in

        let poll_until = Utc::now();
        let code = api.devices_authenticate().map_err(|e| e.to_string())?;

        let poll_until: DateTime<Utc> = poll_until + Duration::seconds(code.expires_in as i64);

        msg.author.direct_message(|m| {
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
        })?;

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

        if tokens.is_none() {
            return Err("Couldn't log you in".to_owned().into());
        }

        let tokens = tokens.unwrap();

        {
            let user = User {
                discord_id: msg.author.id.0 as i64,
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
                expires: Utc.timestamp(tokens.created_at as i64, 0).naive_utc()
                    + Duration::seconds(tokens.expires_in as i64),
            };

            let conn = data
                .get::<Sqlite>()
                .ok_or("Couldn't get SQL connection")?
                .lock()?;

            diesel::insert_into(users::table)
                .values(&user)
                .execute(&*conn)?;
        }

        msg.author.direct_message(|m| {
            m.embed(|e| {
                e.title("Success")
                    .description("You are now logged in. Have fun!")
                    .color((237u8, 28u8, 36u8))
            })
        })?;

        Ok(())
    }
}
