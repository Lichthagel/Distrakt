use diesel::prelude::SqliteConnection;
use std::ops::Deref;
use std::sync::Mutex;
use trakt::TraktApi;

pub struct Trakt {
    api: TraktApi,
}

impl Trakt {
    pub fn new(client_id: String, client_secret: Option<String>) -> Self {
        Self {
            api: TraktApi::new(client_id, client_secret),
        }
    }
}

impl typemap::Key for Trakt {
    type Value = Self;
}

impl Deref for Trakt {
    type Target = TraktApi;

    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

pub struct Sqlite {
    conn: Mutex<SqliteConnection>,
}

impl Sqlite {
    pub fn new(conn: SqliteConnection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }
}

impl typemap::Key for Sqlite {
    type Value = Self;
}

impl Deref for Sqlite {
    type Target = Mutex<SqliteConnection>;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
