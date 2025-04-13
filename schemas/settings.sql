CREATE TABLE IF NOT EXISTS settings (
  id SERIAL PRIMARY KEY,
  logs_ignored_channels BIGINT[] NOT NULL DEFAULT '{}'
);
