use crate::{models::User, schema::users};
use diesel::{prelude::SqliteConnection, query_dsl::QueryDsl, ExpressionMethods, RunQueryDsl};

pub trait UserSql {
    fn get_sql(&self, conn: &SqliteConnection) -> Result<User, String>;
}

impl UserSql for serenity::model::prelude::User {
    fn get_sql(&self, conn: &SqliteConnection) -> Result<User, String> {
        users::dsl::users
            .filter(users::discord_id.eq(self.id.0 as i64))
            .limit(1)
            .load::<User>(conn)
            .map_err(|e| e.to_string())
            .and_then(|mut res: Vec<User>| res.pop().ok_or("Couldn't find user".to_owned()))
    }
}
