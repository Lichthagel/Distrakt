CREATE TABLE users (
  discord_id UNSIGNED BIG INT PRIMARY KEY NOT NULL,
  access_token VARCHAR NOT NULL,
  refresh_token VARCHAR NOT NULL,
  expires DATETIME NOT NULL,
  subscribed BOOLEAN
)