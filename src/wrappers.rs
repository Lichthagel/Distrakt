use sled::Db;
use std::ops::Deref;
use trakt::TraktApi;

pub struct Trakt(TraktApi<'static>);

impl Trakt {
    pub fn new(client_id: String, client_secret: Option<String>) -> Self {
        Self {
            0: TraktApi::new(client_id, client_secret),
        }
    }
}

impl typemap::Key for Trakt {
    type Value = Self;
}

impl Deref for Trakt {
    type Target = TraktApi<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Database(Db);

impl Database {
    pub fn new(db: Db) -> Self {
        Self { 0: db }
    }
}

impl typemap::Key for Database {
    type Value = Self;
}

impl Deref for Database {
    type Target = Db;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
