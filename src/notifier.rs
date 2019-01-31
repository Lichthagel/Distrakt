use crate::{
    schema::{notify, users},
    wrappers::Sqlite,
};
use diesel::{query_dsl::*, NullableExpressionMethods};
use serenity::prelude::RwLock;
use std::{sync::Arc, thread, time::Duration};
use typemap::ShareMap;

pub fn sync_thread(data: Arc<RwLock<ShareMap>>) {
    thread::spawn(move || {
        loop {
            data.read()
                .get::<Sqlite>()
                .ok_or(())
                .and_then(|sql| sql.lock().map_err(|_| ()))
                // Get all notifications
                .and_then(|conn| {
                    notify::dsl::notify
                        .left_join(users::dsl::users)
                        .select((
                            notify::channel,
                            notify::type_,
                            notify::data,
                            users::access_token.nullable(),
                        ))
                        .load::<(i64, i32, Option<i64>, Option<String>)>(&*conn)
                        .map(|res: Vec<(i64, i32, Option<i64>, Option<String>)>| {
                            res.into_iter()
                                .map(|(channel, type_, data, access_token)| {
                                    (
                                        channel as u64,
                                        type_ as u8,
                                        data.map(|i| i as u64),
                                        access_token,
                                    )
                                })
                                .collect()
                        })
                        .map_err(|_| ())
                })
                // (channel_id, notification_type, discord_user_id, access_token)
                .and_then(|d: Vec<(u64, u8, Option<u64>, Option<String>)>| {
                    dbg!(d);
                    Ok(())
                })
                .ok();

            thread::sleep(Duration::from_secs(600))
        }
    });
}
