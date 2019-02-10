CREATE TABLE movies (
  slug VARCHAR NOT NULL,
  released DATE,
  title VARCHAR NOT NULL,
  year INT,
  trakt_id UNSIGNED BIG INT NOT NULL,
  imdb_id VARCHAR,
  tmdb_id UNSIGNED BIG INT,
  tvdb_id UNSIGNED BIG INT,
  tvrage_id UNSIGNED BIG INT,
  CONSTRAINT PK_movie PRIMARY KEY (slug)
);