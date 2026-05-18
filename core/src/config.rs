use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;

use envconfig::Envconfig;
use url::Url;

pub(crate) static APPLICATION_CONFIG: LazyLock<ApplicationConfig> =
    LazyLock::new(|| ApplicationConfig::init_from_env().unwrap());
pub(crate) static CACHE_CONFIG: LazyLock<CacheConfig> = LazyLock::new(|| CacheConfig::init_from_env().unwrap());
pub(crate) static DATABASE_CONFIG: LazyLock<DatabaseConfig> =
    LazyLock::new(|| DatabaseConfig::init_from_env().unwrap());
pub(crate) static MONITOR_CONFIG: LazyLock<MonitorConfig> = LazyLock::new(|| MonitorConfig::init_from_env().unwrap());
pub(crate) static SESSION_CONFIG: LazyLock<SessionConfig> = LazyLock::new(|| SessionConfig::init_from_env().unwrap());
pub(crate) static STORAGE_CONFIG: LazyLock<StorageConfig> = LazyLock::new(|| StorageConfig::init_from_env().unwrap());

#[derive(Envconfig)]
pub(crate) struct ApplicationConfig {
    #[envconfig(from = "APPLICATION_TOKEN_MIN_LENGTH", default = "64")]
    token_min_length: u8,
    #[envconfig(from = "APPLICATION_TOKEN_MAX_LENGTH", default = "128")]
    token_max_length: u8,
    #[envconfig(from = "APPLICATION_TTL_SECS", default = "31104000")]
    ttl_secs: u64,
}

impl ApplicationConfig {
    pub fn token_length(&self) -> RangeInclusive<u8> {
        self.token_min_length..=self.token_max_length
    }

    pub fn ttl(&self) -> Duration {
        Duration::from_secs(self.ttl_secs)
    }
}

#[derive(Envconfig)]
pub(crate) struct CacheConfig {
    #[envconfig(from = "CACHE_REDIS_URL", default = "redis://127.0.0.1:6379/0")]
    pub redis_url: String,
    #[envconfig(from = "CACHE_TTL_SECS", default = "3600")]
    ttl_secs: u64,
}

impl CacheConfig {
    pub fn ttl(&self) -> Duration {
        Duration::from_secs(self.ttl_secs)
    }
}

#[derive(Envconfig)]
pub(crate) struct DatabaseConfig {
    #[envconfig(from = "DATABASE_MAX_CONNECTIONS", default = "5")]
    pub max_connections: u32,
    #[envconfig(from = "DATABASE_URL", default = "postgres://bofe:bofe@127.0.0.1:5432/bofe_dev")]
    pub url: String,
}

#[derive(Envconfig)]
pub(crate) struct MonitorConfig {
    #[envconfig(from = "MONITOR_REDIS_URL", default = "redis://127.0.0.1:6379/1")]
    pub redis_url: String,
}

#[derive(Envconfig)]
pub(crate) struct SessionConfig {
    #[envconfig(from = "SESSION_TOKEN_MIN_LENGTH", default = "64")]
    token_min_length: u8,
    #[envconfig(from = "SESSION_TOKEN_MAX_LENGTH", default = "128")]
    token_max_length: u8,
    #[envconfig(from = "SESSION_TTL_SECS", default = "2592000")]
    ttl_secs: u64,
}

impl SessionConfig {
    pub fn token_length(&self) -> RangeInclusive<u8> {
        self.token_min_length..=self.token_max_length
    }

    pub fn ttl(&self) -> Duration {
        Duration::from_secs(self.ttl_secs)
    }
}

#[derive(Envconfig)]
pub(crate) struct StorageConfig {
    #[envconfig(from = "STORAGE_PATH", default = "./storage/")]
    pub path: PathBuf,
    #[allow(dead_code)]
    #[envconfig(from = "STORAGE_URL", default = "http://127.0.0.1:8005/")]
    pub url: Url,
}
