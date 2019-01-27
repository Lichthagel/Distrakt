CREATE TABLE shows (
  slug VARCHAR,
  title VARCHAR NOT NULL,
  year INT,
  trakt_id UNSIGNED BIG INT,
  imdb_id VARCHAR,
  tmdb_id UNSIGNED BIG INT,
  tvdb_id UNSIGNED BIG INT,
  tvrage_id UNSIGNED BIG INT,
  CONSTRAINT PK_show PRIMARY KEY (slug)
)