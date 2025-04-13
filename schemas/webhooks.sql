CREATE TABLE IF NOT EXISTS webhooks (
  name VARCHAR(100) NOT NULL,
  thread_id VARCHAR(25) NOT NULL,
  message_id VARCHAR(25) NOT NULL,
  id VARCHAR(25) NOT NULL,
  token VARCHAR(255) NOT NULL,
  PRIMARY KEY (id, token),
  FOREIGN KEY (name) REFERENCES mpservers(name) ON DELETE CASCADE
);

ALTER TABLE webhooks DROP COLUMN IF EXISTS guild_id;
ALTER TABLE webhooks DROP CONSTRAINT IF EXISTS webhooks_pkey;
ALTER TABLE webhooks ADD CONSTRAINT webhooks_pkey PRIMARY KEY (id, token);
ALTER TABLE webhooks ADD COLUMN IF NOT EXISTS thread_id VARCHAR(25);
ALTER TABLE webhooks ALTER COLUMN thread_id SET NOT NULL;

CREATE OR REPLACE FUNCTION handle_server_deletion() RETURNS TRIGGER AS $$
DECLARE
BEGIN
  UPDATE webhooks
  SET name = 'server_deleted'
  WHERE name = OLD.name;
  RETURN OLD;
END;
$$ LANGUAGE plpgsql;

DO
$$
DECLARE
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_trigger WHERE tgname = 'server_deletion_trigger'
  ) THEN
    CREATE TRIGGER server_deletion_trigger
    AFTER DELETE ON mpservers
    FOR EACH ROW
    EXECUTE FUNCTION handle_server_deletion();
  END IF;
END;
$$;
