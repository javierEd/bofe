ALTER TABLE cards ADD COLUMN archived_at timestamptz NULL;

ALTER TABLE lists ADD COLUMN archive_cards boolean NOT NULL DEFAULT FALSE;

ALTER TYPE activity_action ADD VALUE 'archive_card' AFTER 'update_card_position';
ALTER TYPE activity_action ADD VALUE 'unarchive_card' AFTER 'archive_card';
