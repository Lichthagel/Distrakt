use crate::distrakt_trakt::Trakt;
use chrono::{DateTime, Utc};
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
        let data = ctx.data.lock();

        let api = data
            .get::<Trakt>()
            .ok_or(CommandError("Couldn't extract api".to_owned()))?;

        // TODO check if already logged in

        let poll_until = Utc::now();
        let code = api
            .devices_authenticate()
            .map_err(|e| e.to_string())?;

        let poll_until: DateTime<Utc> = poll_until + Duration::seconds(code.expires_in as i64);

        msg.author.direct_message(|m| {
            m.content(format!(
                "Go to {} and enter code {} (expires at {})",
                code.verification_url,
                code.user_code,
                poll_until.format("%H:%M:%S UTC")
            ))
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

        // TODO save tokens

        msg.author
            .direct_message(|m| m.content("Successfully logged in"))?;

        Ok(())
    }
}
