ALTER TABLE activities ALTER COLUMN user_id DROP NOT NULL, DROP CONSTRAINT fkey_activities_to_users;

ALTER TABLE activities ADD CONSTRAINT fkey_activities_to_users FOREIGN KEY (user_id) REFERENCES users (
    id
) ON DELETE SET NULL;

ALTER TABLE attachments ALTER COLUMN user_id DROP NOT NULL, DROP CONSTRAINT fkey_attachments_to_users;

ALTER TABLE attachments ADD CONSTRAINT fkey_attachments_to_users FOREIGN KEY (user_id) REFERENCES users (
    id
) ON DELETE SET NULL;

ALTER TABLE cards ALTER COLUMN user_id DROP NOT NULL, DROP CONSTRAINT fkey_cards_to_users;

ALTER TABLE cards ADD CONSTRAINT fkey_cards_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE SET NULL;

ALTER TABLE labels DROP COLUMN user_id;

ALTER TABLE lists DROP COLUMN user_id;
