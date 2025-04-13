CREATE TABLE IF NOT EXISTS sanctions (
  case_id INT PRIMARY KEY,
  case_type VARCHAR(15) NOT NULL,
  member_name VARCHAR(32) NOT NULL,
  member_id VARCHAR(25) NOT NULL,
  moderator_name VARCHAR(32) NOT NULL,
  moderator_id VARCHAR(25) NOT NULL,
  timestamp BIGINT NOT NULL,
  end_time BIGINT,
  duration BIGINT,
  reason VARCHAR(255) NOT NULL
);
