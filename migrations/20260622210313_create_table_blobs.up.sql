CREATE TYPE blob_file_type AS ENUM ('image/gif', 'image/jpeg', 'image/png', 'image/webp');

CREATE TABLE blobs (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    file_type blob_file_type NOT NULL,
    size_bytes bigint NOT NULL,
    sha256_checksum citext NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    CONSTRAINT pkey_blobs PRIMARY KEY (id)
);

CREATE UNIQUE INDEX index_blobs_on_file_type_size_bytes_sha256_checksum ON blobs (
    file_type, size_bytes, sha256_checksum
);
