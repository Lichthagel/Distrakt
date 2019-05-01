use chrono::{DateTime, Utc};
use trakt::models::FullListItem;
use std::ops::{Deref, DerefMut};

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