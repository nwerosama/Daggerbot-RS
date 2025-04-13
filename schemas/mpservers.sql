CREATE TABLE IF NOT EXISTS mpservers (
  name VARCHAR(30) NOT NULL,
  is_active BOOLEAN NOT NULL,
  ip VARCHAR(21) NOT NULL,
  code VARCHAR(32) NOT NULL,
  game_password VARCHAR(16) NOT NULL,
  peak_players INT NOT NULL,
  last_peak_update TIMESTAMP WITH TIME ZONE,
  player_data INT[] NOT NULL,
  PRIMARY KEY (name)
);
