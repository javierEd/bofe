CREATE TABLE attachment_keys (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NULL,
    attachment_id uuid NOT NULL,
    expires_at timestamptz NOT NULL DEFAULT current_timestamp + interval '1 hour',
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    CONSTRAINT pkey_attachment_keys PRIMARY KEY (id),
    CONSTRAINT fkey_attachment_keys_to_users FOREIGN KEY (user_id) REFERENCES users (
        id
    ) ON DELETE CASCADE,
    CONSTRAINT fkey_attachment_keys_to_attachments FOREIGN KEY (attachment_id) REFERENCES attachments (
        id
    ) ON DELETE CASCADE
);
