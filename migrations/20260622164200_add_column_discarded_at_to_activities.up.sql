ALTER TABLE activities ADD COLUMN discarded_at timestamptz NULL, ADD COLUMN updated_at timestamptz NULL;

SELECT manage_updated_at('activities');
