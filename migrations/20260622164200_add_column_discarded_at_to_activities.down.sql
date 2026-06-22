DROP TRIGGER set_updated_at ON activities;

ALTER TABLE activities DROP COLUMN discarded_at, DROP COLUMN updated_at;
