use crate::{
    models::User,
    wrappers::{Trakt, Users},
};
use serenity::{
    client::Context,
    framework::standard::{Args, Command, CommandError},
    model::channel::Message,
};
use std::cmp::min;
use trakt::models::{ListItem, ListItemType};

pub struct Watchlist;

impl Watchlist {
    fn run(ctx: &mut Context, msg: &Message) -> Result<(), String> {
        let lock = ctx.data.read();
        let users = lock
            .get::<Users>()
            .ok_or_else(|| "Couldn't extract users".to_owned())?;

        let user = users
            .get(msg.author.id.0.to_le_bytes())
            .map_err(|e| e.to_string())?;

        drop(lock);

        if let Some(inner) = user {
            let user: User = bincode::deserialize(&inner).map_err(|e| e.to_string())?;

            let lock = ctx.data.read();

            let api = lock
                .get::<Trakt>()
                .ok_or_else(|| "Couldn't extract api".to_owned())?;

            let watchlist: Vec<ListItem> = api
                .sync_watchlist(None, &user.access_token)
                .map_err(|e| e.to_string())?;

            drop(lock);

            let message = watchlist
                .into_iter()
                .map(|item| match item.item_type {
                    ListItemType::Movie => format!(
                        ":movie_camera: [{}] {}",
                        item.rank,
                        item.movie.unwrap().title
                    ),
                    ListItemType::Show => {
                        format!(":film_frames: [{}] {}", item.rank, item.show.unwrap().title)
                    }
                    ListItemType::Season => format!(
                        ":film_frames: [{}] {} Season {}",
                        item.rank,
                        item.show.unwrap().title,
                        item.season.unwrap().number
                    ),
                    ListItemType::Episode => format!(
                        ":film_frames: [{}] {}",
                        item.rank.to_owned(),
                        item.episode
                            .as_ref()
                            .unwrap()
                            .title
                            .to_owned()
                            .unwrap_or_else(|| format!("Episode {}", item.episode.unwrap().number))
                    ),
                    ListItemType::Person => {
                        format!(":mens: [{}] {}", item.rank, item.person.unwrap().name)
                    }
                })
                .collect::<Vec<String>>();

            let name = user.name.unwrap_or(user.username.to_owned());
            let username = user.username;
            let avatar = user.avatar.unwrap_or_default();

            for i in 1..(message.len() / 20) + 2 {
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|embed| {
                            embed
                                .title(":notepad_spiral: W A T C H L I S T")
                                .color((237u8, 28u8, 36u8))
                                .description(
                                    message[((i - 1) * 20)..min(i * 20, message.len())].join("\n"),
                                )
                                .author(|author| {
                                    author
                                        .name(&name)
                                        .url(&format!("https://trakt.tv/users/{}", &username))
                                        .icon_url(&avatar)
                                })
                                .url(&format!("https://trakt.tv/users/{}/watchlist", &username))
                        })
                    })
                    .map_err(|e| e.to_string())?;
            }

            Ok(())
        } else {
            Err("You are not loggen in".to_owned())
        }
    }
}

impl Command for Watchlist {
    fn execute(&self, ctx: &mut Context, msg: &Message, _: Args) -> Result<(), CommandError> {
        let result = Watchlist::run(ctx, msg);

        result.map_err(|e| {
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
