CREATE TABLE users (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    username citext NOT NULL,
    email citext NOT NULL,
    encrypted_password varchar NOT NULL,
    full_name varchar(255) NOT NULL,
    display_name varchar(255) NOT NULL,
    birthdate date NOT NULL,
    language_code varchar(2) NOT NULL DEFAULT 'en',
    country_code varchar(2) NOT NULL,
    disabled_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_users PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_users_on_username ON users USING btree (username) WHERE username != '';
CREATE UNIQUE INDEX index_users_on_email ON users USING btree (email);

SELECT manage_updated_at('users');
