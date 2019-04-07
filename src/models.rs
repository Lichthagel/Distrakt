use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize)]
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
