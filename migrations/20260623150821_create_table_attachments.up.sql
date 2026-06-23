CREATE TABLE attachments (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    blob_id uuid NOT NULL,
    file_name citext NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    CONSTRAINT pkey_attachments PRIMARY KEY (id),
    CONSTRAINT fkey_attachments_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT fkey_attachments_to_blobs FOREIGN KEY (blob_id) REFERENCES blobs (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_attachments_on_user_id_blob_id_file_name ON attachments (user_id, blob_id, file_name);
