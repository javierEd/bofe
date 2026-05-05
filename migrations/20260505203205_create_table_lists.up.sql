CREATE TABLE lists (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    board_id uuid NOT NULL,
    name citext NOT NULL,
    position smallint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_lists PRIMARY KEY (id),
    CONSTRAINT fkey_lists_to_boards FOREIGN KEY (board_id) REFERENCES boards (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_lists_on_board_id_name ON lists USING btree (board_id, name);
CREATE UNIQUE INDEX index_lists_on_board_id_position ON lists USING btree (board_id, position);

SELECT manage_updated_at('lists');
