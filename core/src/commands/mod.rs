use bytesize::ByteSize;
use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;

use toolbox::cache::redis_cache_store;
use toolbox::identity_client::{IdentityClient, IdentityUser};

use crate::config::STORAGE_CONFIG;
use crate::constants::CACHE_PREFIX_GET_IDENTITY_USER;

mod board_commands;
mod card_commands;
mod list_commands;
mod user_commands;

pub use board_commands::*;
pub use card_commands::*;
pub use list_commands::*;
pub use user_commands::*;

pub fn get_available_space() -> ByteSize {
    let stats = uucore::fsext::statfs(STORAGE_CONFIG.path.as_os_str()).expect("Could not get storage stats");

    ByteSize(stats.f_bavail * stats.f_bsize as u64)
}

#[io_cached(
    map_error = r##"|_| anyhow::anyhow!("Could not get identity user")"##,
    convert = r#"{ username_or_id.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, IdentityUser<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_IDENTITY_USER).await }"##
)]
pub async fn get_identity_user(client: &IdentityClient, username_or_id: &str) -> anyhow::Result<IdentityUser<'static>> {
    Ok(client.user(username_or_id).await?)
}
