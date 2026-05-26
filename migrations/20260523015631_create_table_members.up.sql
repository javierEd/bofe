CREATE TABLE members (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    board_id uuid NOT NULL,
    user_id uuid NOT NULL,
    is_admin bool NOT NULL DEFAULT FALSE,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_members PRIMARY KEY (id),
    CONSTRAINT fkey_members_to_boards FOREIGN KEY (board_id) REFERENCES boards (id) ON DELETE CASCADE,
    CONSTRAINT fkey_members_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_memberss_on_board_id_userx_id ON members USING btree (board_id, user_id);

SELECT manage_updated_at('members');
