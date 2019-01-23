use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct DistraktConfig {
    pub prefix: String,
    pub discord_token: String,
    pub trakt_id: String,
    pub trakt_secret: String,
    pub owners: Vec<u64>,
}

impl DistraktConfig {
    pub fn load() -> Self {
        serde_json::from_reader(fs::File::open("config.json").expect("couldn't load config file"))
            .expect("Couldn't load config")
    }
}
