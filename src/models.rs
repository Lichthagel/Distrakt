use chrono::{NaiveDateTime, DateTime, Utc};
use trakt::models::FullListItem;
use std::ops::{Deref, DerefMut};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub discord_id: u64,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: NaiveDateTime,
    pub slug: String,
    pub username: String,
    pub name: Option<String>,
    pub private: bool,
    pub vip: Option<bool>,
    pub cover_image: Option<String>,
    pub avatar: Option<String>,
    pub joined_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Watchlist {
    pub last_downsync: DateTime<Utc>,
    pub list: Vec<FullListItem>,
}

impl Deref for Watchlist {
    type Target = Vec<FullListItem>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for Watchlist {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}