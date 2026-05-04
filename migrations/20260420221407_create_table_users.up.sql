CREATE TABLE users (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    identity_user_id uuid NOT NULL,
    disabled_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_users PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_users_on_identity_user_id ON users USING btree (identity_user_id);

SELECT manage_updated_at('users');
