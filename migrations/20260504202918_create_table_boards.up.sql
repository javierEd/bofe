CREATE TYPE board_visibility AS ENUM ('private', 'users', 'public');

CREATE TABLE boards (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    name citext NOT NULL,
    slug citext NOT NULL,
    description text NOT NULL DEFAULT '',
    visibility board_visibility NOT NULL DEFAULT 'private',
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_boards PRIMARY KEY (id),
    CONSTRAINT fkey_boards_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_boards_on_user_id_name ON boards USING btree (user_id, name);
CREATE INDEX index_boards_on_slug ON boards USING btree (slug);

SELECT manage_updated_at('boards');
