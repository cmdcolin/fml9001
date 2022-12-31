-- Your SQL goes here
CREATE TABLE IF NOT EXISTS tracks (
  filename VARCHAR NOT NULL PRIMARY KEY,
  artist VARCHAR,
  title VARCHAR,
  album VARCHAR,
  genre VARCHAR,
  album_artist VARCHAR,
  track VARCHAR,
  added DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS recently_played (
  filename VARCHAR NOT NULL PRIMARY KEY,
  timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);