CREATE TABLE users (
  discord_id UNSIGNED BIG INT NOT NULL,
  access_token VARCHAR NOT NULL,
  refresh_token VARCHAR NOT NULL,
  expires DATETIME NOT NULL,
  subscribed BOOLEAN,
  CONSTRAINT PK_user PRIMARY KEY (discord_id)
)