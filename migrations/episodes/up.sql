CREATE TABLE episodes (
  trakt_id UNSIGNED BIG INT PRIMARY KEY,
  title VARCHAR NOT NULL,
  season_num INT NOT NULL,
  episode_num INT NOT NULL,
  first_aired DATETIME,
  slug VARCHAR,
  imdb_id VARCHAR,
  tmdb_id UNSIGNED BIG INT,
  tvdb_id UNSIGNED BIG INT,
  tvrage_id UNSIGNED BIG INT
)