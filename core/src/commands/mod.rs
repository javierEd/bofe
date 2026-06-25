use std::borrow::Cow;
use std::fmt::Display;
use std::future::Future;

#[cfg(feature = "graphql")]
use std::hash::{DefaultHasher, Hash, Hasher};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use cached::async_sync::OnceCell;
use cached::{AsyncRedisCache, ConcurrentCachedAsync};
use serde::Serialize;
use serde::de::DeserializeOwned;

#[cfg(feature = "graphql")]
use ab_glyph::{FontRef, PxScale};
#[cfg(feature = "graphql")]
use argon2::PasswordHasher;
#[cfg(feature = "graphql")]
use argon2::password_hash::SaltString;
#[cfg(feature = "graphql")]
use argon2::password_hash::rand_core::OsRng;
#[cfg(feature = "graphql")]
use bytesize::ByteSize;
#[cfg(feature = "graphql")]
use image::{ImageBuffer, Pixel, Rgb, RgbImage};
#[cfg(feature = "graphql")]
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut, text_size};
#[cfg(feature = "graphql")]
use imageproc::rect::Rect;
#[cfg(feature = "graphql")]
use rand::distr::Alphanumeric;
#[cfg(feature = "graphql")]
use rand::distr::uniform::SampleRange;
#[cfg(feature = "graphql")]
use rand::{RngExt, rng};
#[cfg(feature = "graphql")]
use validator::{ValidationError, ValidationErrors};

mod activity_commands;
mod board_commands;
mod card_label_commands;
mod confirmation_commands;
mod im_database_commands;
mod label_commands;
mod list_commands;
mod member_commands;
mod session_commands;
mod user_commands;

#[cfg(feature = "graphql")]
mod application_commands;
#[cfg(feature = "graphql")]
mod attachment_commands;
#[cfg(feature = "graphql")]
mod attachment_key_commands;
#[cfg(feature = "graphql")]
mod blob_commands;
#[cfg(feature = "graphql")]
mod card_commands;
#[cfg(feature = "graphql")]
mod user_email_commands;
#[cfg(feature = "graphql")]
mod user_password_commands;

pub use activity_commands::*;
pub use board_commands::*;
pub(crate) use card_label_commands::*;
pub use confirmation_commands::*;
pub(crate) use im_database_commands::*;
pub(crate) use label_commands::*;
pub(crate) use list_commands::*;
pub(crate) use member_commands::*;
pub use session_commands::*;
pub use user_commands::*;

#[cfg(feature = "graphql")]
pub use application_commands::*;
#[cfg(feature = "graphql")]
pub(crate) use attachment_commands::*;
#[cfg(feature = "graphql")]
pub use attachment_key_commands::*;
#[cfg(feature = "graphql")]
pub(crate) use blob_commands::*;
#[cfg(feature = "graphql")]
pub(crate) use card_commands::*;
#[cfg(feature = "graphql")]
pub(crate) use user_email_commands::*;
#[cfg(feature = "graphql")]
pub(crate) use user_password_commands::*;

use crate::config::CACHE_CONFIG;
use crate::constants::STRIP_MARKDOWN_RULES;

#[cfg(feature = "graphql")]
use crate::config::STORAGE_CONFIG;

#[cfg(feature = "graphql")]
type ValidationResult<T = ()> = Result<T, ValidationErrors>;

trait AsyncRedisCacheExt<K> {
    fn cache_remove(&self, prefix: &str, key: &K) -> impl Future<Output = ()> + Send;
}

impl<K, V> AsyncRedisCacheExt<K> for OnceCell<AsyncRedisCache<K, V>>
where
    K: Clone + Display + Send + Sync,
    V: DeserializeOwned + Send + Serialize,
{
    async fn cache_remove(&self, prefix: &str, key: &K) {
        let _ = self
            .get_or_init(|| async { redis_cache_store(prefix).await })
            .await
            .cache_delete(key)
            .await;
    }
}

#[cfg(feature = "graphql")]
trait OrValidationErrors<T> {
    fn or_validation_errors(self) -> ValidationResult<T>;

    fn or_validation_errors_with(self, field: &'static str, error: ValidationError) -> ValidationResult<T>;
}

#[cfg(feature = "graphql")]
impl<T> OrValidationErrors<T> for Option<T> {
    fn or_validation_errors(self) -> ValidationResult<T> {
        self.ok_or_else(Default::default)
    }

    fn or_validation_errors_with(self, field: &'static str, error: ValidationError) -> ValidationResult<T> {
        let mut validation_errors = ValidationErrors::new();

        validation_errors.add(field, error);

        self.ok_or(validation_errors)
    }
}

#[cfg(feature = "graphql")]
impl<T, E> OrValidationErrors<T> for Result<T, E> {
    fn or_validation_errors(self) -> ValidationResult<T> {
        self.map_err(|_| Default::default())
    }

    fn or_validation_errors_with(self, field: &'static str, error: ValidationError) -> ValidationResult<T> {
        let mut validation_errors = ValidationErrors::new();

        validation_errors.add(field, error);

        self.map_err(|_| validation_errors)
    }
}

#[cfg(feature = "graphql")]
fn get_available_space() -> ByteSize {
    let stats = uucore::fsext::statfs(STORAGE_CONFIG.path.as_os_str()).expect("Could not get storage stats");

    ByteSize(stats.f_bavail * stats.f_bsize as u64)
}

pub(crate) fn markdown_to_text(input: &str) -> String {
    let mut text = Cow::Borrowed(input);

    for (regex, replacement) in STRIP_MARKDOWN_RULES.iter() {
        text = Cow::Owned(regex.replace_all(&text, *replacement).into_owned());
    }

    text.into_owned()
}

#[cfg(feature = "graphql")]
fn encrypt_password(value: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(value.as_bytes(), &salt).unwrap().to_string()
}

#[cfg(feature = "graphql")]
fn random_numeric_string(length: u8) -> String {
    let mut rng = rng();

    (0..length).map(|_| rng.random_range(0..=9).to_string()).collect()
}

#[cfg(feature = "graphql")]
fn random_string<R: SampleRange<u8>>(length: R) -> String {
    let mut rng = rng();

    let length = rng.random_range(length);

    rng.sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect()
}

async fn redis_cache_store<K, V>(prefix: &str) -> AsyncRedisCache<K, V>
where
    K: Display + Send + Sync,
    V: DeserializeOwned + Send + Serialize,
{
    AsyncRedisCache::new(format!("{prefix}:"), CACHE_CONFIG.ttl())
        .connection_string(&CACHE_CONFIG.redis_url)
        .refresh(true)
        .build()
        .await
        .expect("Could not get redis cache")
}

#[cfg(feature = "graphql")]
fn text_icon(text: &str, size: u16) -> anyhow::Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
    let text_initials = &text[0..2].to_uppercase();
    let size = size as u32;

    let background_color = text_to_rgb(text);
    let mut text_color = background_color;

    text_color.invert();

    let mut rgb_image = RgbImage::new(size, size);

    draw_filled_rect_mut(&mut rgb_image, Rect::at(0, 0).of_size(size, size), background_color);

    let font_file = std::fs::read(&STORAGE_CONFIG.font_path)?;
    let font = FontRef::try_from_slice(&font_file)?;
    let scale = PxScale::from(size as f32 / 1.7);
    let (text_width, _) = text_size(scale, &font, text_initials);
    let x = ((size - text_width) / 2) as i32;
    let y = (size as f32 / 4.6) as i32;

    draw_text_mut(&mut rgb_image, text_color, x, y, scale, &font, text_initials);

    Ok(rgb_image)
}

#[cfg(feature = "graphql")]
fn text_to_rgb(text: &str) -> Rgb<u8> {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash_value = hasher.finish();

    let red = (hash_value & 0xFF) as u8;
    let green = ((hash_value >> 8) & 0xFF) as u8;
    let blue = ((hash_value >> 16) & 0xFF) as u8;

    Rgb([red, green, blue])
}

pub(crate) fn verify_password(encrypted_password: &str, password: &str) -> bool {
    let argon2 = Argon2::default();

    let Ok(password_hash) = PasswordHash::new(encrypted_password) else {
        return false;
    };

    argon2.verify_password(password.as_bytes(), &password_hash).is_ok()
}
