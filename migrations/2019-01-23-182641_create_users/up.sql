CREATE TABLE users (
  discord_id INT(64) PRIMARY KEY NOT NULL,
  access_token VARCHAR NOT NULL,
  refresh_token VARCHAR NOT NULL,
  expires DATE NOT NULL
)main