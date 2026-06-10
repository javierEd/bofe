use std::borrow::Cow;
use std::fmt::Display;
use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::commands;
use crate::config::STORAGE_CONFIG;
use crate::enums::{CountryCode, LanguageCode};

#[derive(Clone, Deserialize, Serialize)]
pub struct User<'a> {
    pub id: Uuid,
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub email_confirmed_at: Option<DateTime<Utc>>,
    pub(crate) encrypted_password: Cow<'a, str>,
    pub full_name: Cow<'a, str>,
    pub display_name: Cow<'a, str>,
    pub birthdate: NaiveDate,
    pub language_code: LanguageCode,
    pub country_code: CountryCode,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for User<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl User<'_> {
    pub fn avatar_image(&self, size: u16) -> anyhow::Result<Vec<u8>> {
        commands::get_user_avatar_image(self, size)
    }

    pub(crate) fn avatar_image_path(&self, size: u16) -> PathBuf {
        STORAGE_CONFIG
            .path
            .join(format!("users/{}/avatar-image/{size}x{size}.jpg", self.id))
    }

    pub fn avatar_image_url(&self) -> Url {
        STORAGE_CONFIG
            .url
            .join(&format!("users/{}/avatar-image", self.id))
            .unwrap()
    }

    pub(crate) fn email_is_confirmed(&self) -> bool {
        self.email_confirmed_at.is_some()
    }

    pub(crate) fn initials(&self) -> String {
        self.username[0..2].to_uppercase()
    }

    pub(crate) fn verify_password(&self, password: &str) -> bool {
        commands::verify_password(&self.encrypted_password, password)
    }
}
