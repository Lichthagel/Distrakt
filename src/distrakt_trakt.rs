use std::ops::Deref;
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
