CREATE TABLE notifications (
  channel UNSIGNED BIG INT NOT NULL,
  trakt_id UNSIGNED BIG INT NOT NULL,
  CONSTRAINT PK_notifications PRIMARY KEY (channel, trakt_id)
)