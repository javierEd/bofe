use std::borrow::Cow;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands;
use crate::config::STORAGE_CONFIG;
use crate::enums::BlobFileType;

#[derive(Clone, Deserialize, Serialize)]
pub struct Blob<'a> {
    pub id: Uuid,
    pub file_type: BlobFileType,
    pub size_bytes: i64,
    pub sha256_checksum: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
}

impl Blob<'_> {
    pub fn directory(&self) -> PathBuf {
        STORAGE_CONFIG.path.join(format!("blobs/{}/", self.id))
    }

    pub fn path(&self) -> PathBuf {
        self.directory().join(format!("default.{}", self.file_type.extension()))
    }

    pub fn thumbnail_file_type(&self) -> BlobFileType {
        match self.file_type {
            BlobFileType::ImagePng | BlobFileType::ImageSvgXml => BlobFileType::ImagePng,
            BlobFileType::ImageWebp => BlobFileType::ImageWebp,
            _ => BlobFileType::ImageJpeg,
        }
    }

    pub fn thumbnail_path(&self, width: u16, height: u16) -> PathBuf {
        self.directory().join(format!(
            "thumbnail-{width}x{height}.{}",
            self.thumbnail_file_type().extension()
        ))
    }

    pub fn read(&self) -> std::io::Result<Vec<u8>> {
        std::fs::read(self.path())
    }

    pub fn read_thumbnail(&self, width: u16, height: u16) -> anyhow::Result<Vec<u8>> {
        commands::get_or_create_blob_thumbnail(self, width, height)
    }
}
