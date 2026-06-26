use std::borrow::Cow;

#[cfg(feature = "graphql")]
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "graphql")]
use url::Url;

use crate::commands;
use crate::enums::{ConfirmationAction, CountryCode};

#[cfg(feature = "graphql")]
use crate::enums::BlobFileType;

#[cfg(feature = "graphql")]
use crate::config::STORAGE_CONFIG;

mod activity;
mod board;
mod card;
mod label;
mod list;
mod user;

pub use activity::{Activity, ActivityExt};
pub(crate) use board::Board;
pub(crate) use card::Card;
pub(crate) use label::Label;
pub(crate) use list::List;
pub use user::User;

#[derive(Clone, Deserialize, Serialize)]
pub struct Application<'a> {
    pub id: Uuid,
    pub name: Cow<'a, str>,
    pub token: Cow<'a, str>,
    pub expires_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "graphql")]
#[derive(Clone, Deserialize, Serialize)]
pub struct Attachment<'a> {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub blob_id: Uuid,
    pub file_name: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "graphql")]
impl Attachment<'_> {
    pub async fn blob(&self) -> sqlx::Result<Blob<'_>> {
        commands::get_blob_by_id(self.blob_id).await
    }

    pub fn file_name_without_extension(&self) -> &str {
        self.file_name.split('.').collect::<Vec<&str>>()[0]
    }

    pub async fn file_type(&self) -> sqlx::Result<BlobFileType> {
        Ok(self.blob().await?.file_type)
    }

    pub async fn read(&self) -> std::io::Result<Vec<u8>> {
        self.blob()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .read()
    }

    pub async fn url(&self, user: Option<&User<'_>>) -> anyhow::Result<Url> {
        let attachment_key = commands::insert_attachment_key(user, self).await?;
        let file_url = STORAGE_CONFIG.url.join(&format!("attachments/{}", attachment_key.id))?;

        Ok(file_url)
    }
}

#[cfg(feature = "graphql")]
#[derive(Clone, Deserialize, Serialize)]
pub struct AttachmentKey {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub attachment_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "graphql")]
impl AttachmentKey {
    pub async fn attachment(&self) -> sqlx::Result<Attachment<'_>> {
        commands::get_attachment_by_id(self.attachment_id).await
    }
}

#[cfg(feature = "graphql")]
#[derive(Clone, Deserialize, Serialize)]
pub struct Blob<'a> {
    pub id: Uuid,
    pub file_type: BlobFileType,
    pub size_bytes: i64,
    pub sha256_checksum: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "graphql")]
impl Blob<'_> {
    pub fn directory(&self) -> PathBuf {
        STORAGE_CONFIG.path.join(format!("blobs/{}", self.id))
    }

    pub fn path(&self) -> PathBuf {
        self.directory().join(format!("default.{}", self.file_type.extension()))
    }

    pub fn thumbnail_file_type(&self) -> BlobFileType {
        match self.file_type {
            BlobFileType::ImagePng => BlobFileType::ImagePng,
            BlobFileType::ImageWebp => BlobFileType::ImageWebp,
            _ => BlobFileType::ImageJpeg,
        }
    }

    pub fn thumbnail_path(&self, width: u16, height: u16) -> PathBuf {
        self.directory()
            .join(format!("thumbnail-{width}x{height}.{}", self.file_type.extension()))
    }

    pub fn read(&self) -> std::io::Result<Vec<u8>> {
        std::fs::read(self.path())
    }

    pub fn read_thumbnail(&self, width: u16, height: u16) -> anyhow::Result<Vec<u8>> {
        commands::get_or_create_blob_thumbnail(self, width, height)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct CardLabel {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    pub card_id: Uuid,
    pub label_id: Uuid,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl CardLabel {
    pub async fn label<'a>(&self) -> sqlx::Result<Label<'a>> {
        commands::get_label_by_id(self.label_id).await
    }
}

#[derive(Clone)]
pub struct Confirmation<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: ConfirmationAction,
    pub(crate) encrypted_code: Cow<'a, str>,
    pub pending_attempts: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Confirmation<'_> {
    pub async fn user(&self) -> sqlx::Result<User<'_>> {
        commands::get_user_by_id(self.user_id).await
    }

    pub fn verify_code(&self, code: &str) -> bool {
        commands::verify_password(&self.encrypted_code, code)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct Member {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "graphql")]
impl Member {
    pub async fn board<'a>(&self) -> sqlx::Result<Board<'a>> {
        commands::get_board_by_id(self.board_id).await
    }

    pub async fn user<'a>(&self) -> sqlx::Result<User<'a>> {
        commands::get_user_by_id(self.user_id).await
    }

    /// Returns true if the user can edit the member
    ///
    /// Only the owner of the board can edit the member
    pub async fn is_editable(&self, user: &User<'_>) -> sqlx::Result<bool> {
        Ok(self.board().await?.is_editable(user))
    }

    /// Returns true if the user can remove the member
    ///
    /// Only the owner of the board or the same use can remove the member
    pub async fn is_removable(&self, user: &User<'_>) -> sqlx::Result<bool> {
        Ok(self.user_id == user.id || self.is_editable(user).await?)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Session<'a> {
    pub id: Uuid,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub token: Cow<'a, str>,
    pub ip_address: Cow<'a, str>,
    pub country_code: Option<CountryCode>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Session<'_> {
    pub fn location(&self) -> String {
        let Some(country) = self.country_code else {
            return "Unknown".to_owned();
        };

        let mut location = country.name().to_owned();

        if let Some(region) = &self.region {
            location += &format!(", {region}");
        }

        if let Some(city) = &self.city {
            location += &format!(", {city}");
        }

        location
    }

    pub async fn user<'a>(&self) -> sqlx::Result<User<'a>> {
        commands::get_user_by_id(self.user_id).await
    }
}
