use super::schema::users;
use chrono::NaiveDateTime;

#[derive(Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    pub discord_id: i64,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: NaiveDateTime,
    pub subscribed: bool
}
