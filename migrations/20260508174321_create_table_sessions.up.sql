CREATE TABLE sessions (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    application_id uuid NOT NULL,
    user_id uuid NOT NULL,
    token citext NOT NULL,
    ip_address varchar(255) NOT NULL,
    country_code varchar(2) NULL,
    region varchar(255) NULL,
    city varchar(255) NULL,
    expires_at timestamptz NOT NULL DEFAULT current_timestamp + interval '30 days',
    refreshed_at timestamptz NULL,
    finished_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_sessions PRIMARY KEY (id),
    CONSTRAINT fkey_sessions_to_applications FOREIGN KEY (application_id) REFERENCES applications (
        id
    ) ON DELETE CASCADE,
    CONSTRAINT fkey_sessions_to_users FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_sessions_on_token ON sessions USING btree (token);

SELECT manage_updated_at('sessions');
