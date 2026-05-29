use std::fmt::Display;
use std::future::Future;
use std::hash::{DefaultHasher, Hash, Hasher};

use ab_glyph::{FontRef, PxScale};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use bytesize::ByteSize;
use cached::async_sync::OnceCell;
use cached::{AsyncRedisCache, ConcurrentCachedAsync};
use image::{ImageBuffer, Pixel, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut, text_size};
use imageproc::rect::Rect;
use rand::distr::Alphanumeric;
use rand::distr::uniform::SampleRange;
use rand::{RngExt, rng};
use serde::Serialize;
use serde::de::DeserializeOwned;
use validator::ValidationErrors;

mod application_commands;
mod board_channel_commands;
mod board_commands;
mod card_commands;
mod list_commands;
mod member_commands;
mod session_commands;
mod user_commands;

pub use application_commands::*;
pub(crate) use board_channel_commands::*;
pub(crate) use board_commands::*;
pub(crate) use card_commands::*;
pub(crate) use list_commands::*;
pub(crate) use member_commands::*;
pub use session_commands::*;
pub use user_commands::*;

use crate::config::{CACHE_CONFIG, STORAGE_CONFIG};

type ValidationResult<T = ()> = Result<T, ValidationErrors>;

trait AsyncRedisCacheExt<K> {
    fn cache_remove(&self, prefix: &str, key: &K) -> impl Future<Output = ()> + Send;
}

impl<K, V> AsyncRedisCacheExt<K> for OnceCell<AsyncRedisCache<K, V>>
where
    K: Display + Send + Sync,
    V: DeserializeOwned + Display + Send + Serialize + Sync,
{
    async fn cache_remove(&self, prefix: &str, key: &K) {
        let _ = self
            .get_or_init(|| async { redis_cache_store(prefix).await })
            .await
            .cache_delete(key)
            .await;
    }
}

trait OrValidationErrors<T> {
    fn or_validation_errors(self) -> ValidationResult<T>;
}

impl<T> OrValidationErrors<T> for Option<T> {
    fn or_validation_errors(self) -> ValidationResult<T> {
        self.ok_or_else(Default::default)
    }
}

impl<T, E> OrValidationErrors<T> for Result<T, E> {
    fn or_validation_errors(self) -> ValidationResult<T> {
        self.map_err(|_| Default::default())
    }
}

#[allow(dead_code)]
fn get_available_space() -> ByteSize {
    let stats = uucore::fsext::statfs(STORAGE_CONFIG.path.as_os_str()).expect("Could not get storage stats");

    ByteSize(stats.f_bavail * stats.f_bsize as u64)
}

fn encrypt_password(value: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(value.as_bytes(), &salt).unwrap().to_string()
}

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
    V: DeserializeOwned + Display + Send + Serialize + Sync,
{
    AsyncRedisCache::new(format!("{prefix}:"), CACHE_CONFIG.ttl())
        .connection_string(&CACHE_CONFIG.redis_url)
        .refresh(true)
        .build()
        .await
        .expect("Could not get redis cache")
}

pub(crate) fn text_icon(text: &str, size: u16) -> anyhow::Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
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
