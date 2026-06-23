use std::fs::File;
use std::io::BufReader;

use bytesize::ByteSize;
use digest_io::IoWrapper;
use sha2::{Digest, Sha256};

use crate::db_pool;
use crate::enums::BlobFileType;
use crate::models::Blob;

use super::get_available_space;

pub async fn get_or_insert_blob(file: &File) -> sqlx::Result<Blob<'_>> {
    let mut reader = BufReader::new(file);
    let mut hasher = IoWrapper(Sha256::new());

    std::io::copy(&mut reader, &mut hasher)?;

    let file_type = BlobFileType::try_from(file)?;
    let size_bytes = file.metadata()?.len();
    let sha256_checksum = format!("{:?}", hasher.0.finalize());

    let db_pool = db_pool().await;

    if let Ok(blob) = sqlx::query_as!(
        Blob,
        r#"SELECT id, file_type  AS "file_type!: BlobFileType", size_bytes, sha256_checksum, created_at FROM blobs
        WHERE file_type = $1 AND size_bytes = $2 AND sha256_checksum = $3"#,
        file_type as _,    // $1
        size_bytes as i64, // $2
        sha256_checksum,   // $3
    )
    .fetch_one(db_pool)
    .await
    {
        return Ok(blob);
    }

    if get_available_space() <= ByteSize(size_bytes) {
        return Err(sqlx::Error::Io(std::io::Error::other("Not enough space")));
    }

    let mut transaction = db_pool.begin().await?;

    let blob = sqlx::query_as!(
        Blob,
        r#"INSERT INTO blobs (file_type, size_bytes, sha256_checksum, created_at) VALUES ($1, $2, $3, NOW())
        RETURNING id, file_type AS "file_type!: BlobFileType", size_bytes, sha256_checksum, created_at"#,
        file_type as _,    // $1
        size_bytes as i64, // $2
        sha256_checksum,   // $3
    )
    .fetch_one(&mut *transaction)
    .await?;

    std::fs::create_dir_all(blob.directory())?;

    let mut dest_file = File::create(blob.path())?;

    std::io::copy(&mut reader, &mut dest_file)?;

    transaction.commit().await?;

    Ok(blob)
}
