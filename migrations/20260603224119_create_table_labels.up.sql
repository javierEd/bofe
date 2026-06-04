CREATE TABLE labels (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    board_id uuid NOT NULL,
    user_id uuid NOT NULL,
    name citext NOT NULL,
    color_code varchar(7) NOT NULL DEFAULT '#999',
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_labels PRIMARY KEY (id),
    CONSTRAINT fkey_labels_to_boards FOREIGN KEY (board_id) REFERENCES boards (id) ON DELETE CASCADE,
    CONSTRAINT fkey_labels_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_labels_on_board_id_name ON labels USING btree (board_id, name);

SELECT manage_updated_at('labels');
