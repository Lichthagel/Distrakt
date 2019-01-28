use crate::{
    schema::{notify, users},
    wrappers::Sqlite,
};
use diesel::{query_dsl::*, NullableExpressionMethods};
use futures::future::IntoFuture;
use serenity::prelude::Context;

pub fn sync_thread(ctx: Context) {
    let fut = ctx
        .data
        .read()
        .get::<Sqlite>()
        .ok_or(())
        .and_then(|conn| conn.lock().map_err(|_| ()))
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
                .map_err(|_| ())
        })
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
        .and_then(|d: Vec<(u64, u8, Option<u64>, Option<String>)>| {
            dbg!(d);
            Ok(())
        });

    tokio::run(fut.into_future());
}
