use std::fmt::Display;
use std::future::Future;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use bytesize::ByteSize;
use cached::async_sync::OnceCell;
use cached::{AsyncRedisCache, ConcurrentCachedAsync};
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
mod session_commands;
mod user_commands;

pub use application_commands::*;
pub(crate) use board_channel_commands::*;
pub(crate) use board_commands::*;
pub(crate) use card_commands::*;
pub(crate) use list_commands::*;
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

pub(crate) fn verify_password(encrypted_password: &str, password: &str) -> bool {
    let argon2 = Argon2::default();

    let Ok(password_hash) = PasswordHash::new(encrypted_password) else {
        return false;
    };

    argon2.verify_password(password.as_bytes(), &password_hash).is_ok()
}
