CREATE TABLE applications (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    name citext NOT NULL,
    token citext NOT NULL,
    expires_at timestamptz NOT NULL DEFAULT current_timestamp + interval '1 year',
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_applications PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_applications_on_name ON applications USING btree (name);
CREATE UNIQUE INDEX index_applications_on_token ON applications USING btree (token);

SELECT manage_updated_at('applications');
