use std::borrow::Cow;
use std::fmt::Display;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands;
use crate::config::STORAGE_CONFIG;
use crate::enums::{BlobFileType, ConfirmationAction, CountryCode};

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

impl Display for Application<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[cfg(feature = "graphql")]
pub(crate) struct Attachment<'a> {
    pub id: Uuid,
    #[allow(dead_code)]
    pub user_id: Uuid,
    #[allow(dead_code)]
    pub blob_id: Uuid,
    #[allow(dead_code)]
    pub file_name: Cow<'a, str>,
    pub created_at: DateTime<Utc>,
}

pub(crate) struct Blob<'a> {
    pub id: Uuid,
    pub file_type: BlobFileType,
    #[allow(dead_code)]
    pub size_bytes: i64,
    #[allow(dead_code)]
    pub sha256_checksum: Cow<'a, str>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl Blob<'_> {
    pub fn directory(&self) -> PathBuf {
        STORAGE_CONFIG.path.join(format!("blobs/{}", self.id))
    }

    pub fn path(&self) -> PathBuf {
        self.directory().join(format!("default.{}", self.file_type.extension()))
    }

    #[allow(dead_code)]
    pub fn read(&self) -> std::io::Result<Vec<u8>> {
        std::fs::read(self.path())
    }
}

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

impl Display for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
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

impl Display for Session<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
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
