use std::fs::File;
use std::io::BufReader;

use bytesize::ByteSize;
use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use digest_io::IoWrapper;
use image::imageops::FilterType;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::constants::CACHE_PREFIX_GET_BLOB_BY_ID;
use crate::db_pool;
use crate::enums::BlobFileType;
use crate::models::Blob;

use super::{get_available_space, redis_cache_store};

pub fn get_or_create_blob_thumbnail(blob: &Blob, width: u16, height: u16) -> anyhow::Result<Vec<u8>> {
    if !(2..=1024).contains(&width)
        || (width & (width - 1) != 0)
        || !(2..=1024).contains(&height)
        || (height & (height - 1) != 0)
    {
        return Err(anyhow::anyhow!(
            "Invalid thumbnail dimensions: width={width} height={height}"
        ));
    }

    let thumbnail_path = blob.thumbnail_path(width, height);

    if thumbnail_path.exists() {
        return Ok(std::fs::read(&thumbnail_path)?);
    }

    let mut image_decoder = ImageReader::open(blob.path())?.into_decoder()?;

    let (img_width, img_height) = image_decoder.dimensions();

    let max_width = std::cmp::min(width as u32, img_width);
    let max_height = std::cmp::min(height as u32, img_height);
    let orientation = image_decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut dynamic_image = DynamicImage::from_decoder(image_decoder)?;

    dynamic_image.apply_orientation(orientation);

    let image_format = match blob.thumbnail_file_type() {
        BlobFileType::ImagePng => ImageFormat::Png,
        BlobFileType::ImageWebp => ImageFormat::WebP,
        _ => ImageFormat::Jpeg,
    };

    dynamic_image
        .resize(max_width, max_height, FilterType::CatmullRom)
        .save_with_format(&thumbnail_path, image_format)?;

    Ok(std::fs::read(&thumbnail_path)?)
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Blob<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_BLOB_BY_ID).await }"##
)]
pub async fn get_blob_by_id(id: Uuid) -> sqlx::Result<Blob<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Blob,
        r#"SELECT id, file_type  AS "file_type!: BlobFileType", size_bytes, sha256_checksum, created_at
        FROM blobs WHERE id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

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
        r#"SELECT id, file_type  AS "file_type!: BlobFileType", size_bytes, sha256_checksum, created_at
        FROM blobs WHERE file_type = $1 AND size_bytes = $2 AND sha256_checksum = $3"#,
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
