use crate::distrakt_trakt::Trakt;
use serenity::{
    framework::standard::{Args, Command, CommandError},
    model::channel::Message,
    prelude::Context,
};
use std::{thread, time::Duration};
use trakt::error::Error;

pub struct Login;

impl Command for Login {
    fn execute(&self, ctx: &mut Context, msg: &Message, _args: Args) -> Result<(), CommandError> {
        let data = ctx.data.lock();

        let api = data
            .get::<Trakt>()
            .ok_or(CommandError("Couldn't extract api".to_owned()))?;

        let code = api
            .devices_authenticate()
            .map_err(|e| CommandError(e.to_string()))?;

        msg.author.direct_message(|m| {
            m.content(format!(
                "Go to {} and enter code {}",
                code.verification_url, code.user_code
            ))
        })?;

        let mut tokens = None;

        {
            let mut n = 0;
            'poll: while {
                thread::sleep(Duration::from_secs(code.interval));

                let res = api.get_token(&code.device_code);
                n += code.interval;

                match match res {
                    Ok(body) => Ok(Some(body)),
                    Err(e) => {
                        if let Error::Response(res) = e {
                            if res.status().as_u16() != 400 {
                                Err(CommandError(res.status().to_string()))
                            } else {
                                Ok(None)
                            }
                        } else {
                            Err(CommandError(e.to_string()))
                        }
                    }
                }? {
                    Some(body) => {
                        tokens = Some(body);
                        break 'poll;
                    }
                    None => {}
                };

                n < code.expires_in // TODO actually use current time instead of counting
            } {}
        }

        if tokens.is_none() {
            return Err(CommandError("Couldn't log you in".to_owned()));
        }

        let tokens = tokens.unwrap();

        // TODO save tokens

        msg.author
            .direct_message(|m| m.content("Successfully logged in"))?;

        Ok(())
    }
}
