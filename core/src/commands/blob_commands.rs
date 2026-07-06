use std::process::Command;

use bytes::Bytes;
use bytesize::ByteSize;
use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use image::imageops::FilterType;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};
use resvg::tiny_skia::Pixmap;
use resvg::usvg::{self, Transform};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::STORAGE_CONFIG;
use crate::constants::CACHE_PREFIX_GET_BLOB_BY_ID;
use crate::db_pool;
use crate::enums::BlobFileType;
use crate::models::Blob;

use super::{get_available_space, redis_cache_store};

pub fn get_or_create_blob_thumbnail(blob: &Blob, width: u16, height: u16) -> anyhow::Result<Vec<u8>> {
    let thumbnail_path = blob.thumbnail_path(width, height);

    if thumbnail_path.exists() {
        return Ok(std::fs::read(&thumbnail_path)?);
    }

    if !(2..=1024).contains(&width)
        || (width & (width - 1) != 0)
        || !(2..=1024).contains(&height)
        || (height & (height - 1) != 0)
    {
        return Err(anyhow::anyhow!(
            "Invalid thumbnail dimensions: width={width} height={height}"
        ));
    }

    match blob.file_type {
        BlobFileType::ApplicationPdf => create_blob_document_thumbnail(blob, width, height),
        BlobFileType::ImageGif | BlobFileType::ImageJpeg | BlobFileType::ImagePng | BlobFileType::ImageWebp => {
            create_blob_image_thumbnail(blob, width, height)
        }
        BlobFileType::ImageSvgXml => create_blob_image_svg_thumbnail(blob, width, height),
        BlobFileType::VideoMp4 | BlobFileType::VideoOgg | BlobFileType::VideoWebm => {
            create_blob_video_thumbnail(blob, width, height)
        }
    }?;

    Ok(std::fs::read(&thumbnail_path)?)
}

fn create_blob_document_thumbnail(blob: &Blob, width: u16, height: u16) -> anyhow::Result<()> {
    let blob_path = blob.path();
    let source_path = blob_path.to_string_lossy();

    let pdfinfo_output = Command::new("pdfinfo")
        .arg(source_path.to_string())
        .output()
        .unwrap()
        .stdout;

    let pdfinfo_str = String::from_utf8(pdfinfo_output)?;

    let mut page_size = pdfinfo_str
        .lines()
        .find(|line| line.starts_with("Page size:"))
        .ok_or_else(|| anyhow::anyhow!("Could not get page size"))?
        .trim_start_matches("Page size: ")
        .trim()
        .split(" ");
    let page_width = page_size
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not get page width"))?
        .parse::<f32>()?;
    let page_height = page_size
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Could not parse page height"))?
        .parse::<f32>()?;
    let max_width = page_width.min(width as f32);
    let max_height = page_height.min(height as f32);
    let page_aspect_ratio = page_width / page_height;
    let cur_aspect_ratio = max_width / max_height;
    let (max_width, max_height) = if cur_aspect_ratio > page_aspect_ratio {
        ((max_height / page_height * page_width) as u16, max_height as u16)
    } else if cur_aspect_ratio < page_aspect_ratio {
        (max_width as u16, (max_width / page_width * page_height) as u16)
    } else {
        (max_width as u16, max_height as u16)
    };

    let thumbnail_path = blob.thumbnail_path(width, height);

    let _ = Command::new("pdftoppm")
        .args([
            "-f",
            "1",
            "-l",
            "1",
            "-scale-to-x",
            &max_width.to_string(),
            "-scale-to-y",
            &max_height.to_string(),
            "-jpeg",
            "-singlefile",
            &source_path,
            thumbnail_path.to_string_lossy().trim_end_matches(".jpg"),
        ])
        .output()?;

    Ok(())
}

fn create_blob_image_thumbnail(blob: &Blob, width: u16, height: u16) -> anyhow::Result<()> {
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

    let thumbnail_path = blob.thumbnail_path(width, height);

    dynamic_image
        .resize(max_width, max_height, FilterType::CatmullRom)
        .save_with_format(&thumbnail_path, image_format)?;

    Ok(())
}

fn create_blob_image_svg_thumbnail(blob: &Blob, width: u16, height: u16) -> anyhow::Result<()> {
    let svg_data = blob.read()?;

    let tree = usvg::Tree::from_data(&svg_data, &usvg::Options::default())?;
    let mut pixmap = Pixmap::new(width as u32, height as u32).expect("Could not create pixmap");

    resvg::render(
        &tree,
        Transform::from_scale(width as f32 / tree.size().width(), height as f32 / tree.size().height()),
        &mut pixmap.as_mut(),
    );

    let thumbnail_path = blob.thumbnail_path(width, height);

    pixmap.save_png(&thumbnail_path)?;

    Ok(())
}

fn create_blob_video_thumbnail(blob: &Blob, width: u16, height: u16) -> anyhow::Result<()> {
    let blob_path = blob.path();
    let source_path = blob_path.to_string_lossy();

    let duration_out = Command::new("ffprobe")
        .args([
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &source_path,
        ])
        .output()?
        .stdout;
    let duration_str = String::from_utf8(duration_out)?;
    let duration = duration_str.trim().parse::<f32>()?;
    let specified_time = (duration / 2.0).to_string();
    let thumbnail_path = blob.thumbnail_path(width, height);

    let _ = Command::new("ffmpeg")
        .args([
            "-i",
            &source_path,
            "-vf",
            format!(
                "scale='min({},iw):min({},ih):force_original_aspect_ratio=decrease'",
                width, height
            )
            .as_str(),
            "-vframes",
            "1",
            "-update",
            "true",
            "-ss",
            &specified_time,
            &thumbnail_path.to_string_lossy(),
        ])
        .output()?;

    Ok(())
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Blob<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_BLOB_BY_ID).await }"##
)]
pub async fn get_blob_by_id<'a>(id: Uuid) -> sqlx::Result<Blob<'a>> {
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

pub async fn get_or_insert_blob(content: &Bytes) -> sqlx::Result<Blob<'_>> {
    let mut hasher = Sha256::new();

    hasher.update(content);

    let hash_bytes = hasher.finalize();
    let sha256_checksum: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    let file_type = BlobFileType::try_from(content)?;
    let size_bytes = content.len() as i64;

    let db_pool = db_pool().await;

    if let Ok(blob) = sqlx::query_as!(
        Blob,
        r#"SELECT id, file_type  AS "file_type!: BlobFileType", size_bytes, sha256_checksum, created_at
        FROM blobs WHERE file_type = $1 AND size_bytes = $2 AND sha256_checksum = $3"#,
        file_type as _,  // $1
        size_bytes,      // $2
        sha256_checksum, // $3
    )
    .fetch_one(db_pool)
    .await
    {
        return Ok(blob);
    }

    let size = ByteSize(size_bytes as u64);

    if size > STORAGE_CONFIG.max_file_size() || size > get_available_space() {
        return Err(sqlx::Error::Io(std::io::Error::other(
            "File size exceeds maximum allowed size or insufficient storage",
        )));
    }

    let mut transaction = db_pool.begin().await?;

    let blob = sqlx::query_as!(
        Blob,
        r#"INSERT INTO blobs (file_type, size_bytes, sha256_checksum, created_at) VALUES ($1, $2, $3, NOW())
        RETURNING id, file_type AS "file_type!: BlobFileType", size_bytes, sha256_checksum, created_at"#,
        file_type as _,  // $1
        size_bytes,      // $2
        sha256_checksum, // $3
    )
    .fetch_one(&mut *transaction)
    .await?;

    std::fs::create_dir_all(blob.directory())?;

    std::fs::write(blob.path(), content)?;

    transaction.commit().await?;

    Ok(blob)
}
