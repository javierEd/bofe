use std::path::PathBuf;
use std::sync::LazyLock;

use envconfig::Envconfig;
use url::Url;

pub(crate) static DATABASE_CONFIG: LazyLock<DatabaseConfig> =
    LazyLock::new(|| DatabaseConfig::init_from_env().unwrap());
pub(crate) static MONITOR_CONFIG: LazyLock<MonitorConfig> = LazyLock::new(|| MonitorConfig::init_from_env().unwrap());
pub(crate) static STORAGE_CONFIG: LazyLock<StorageConfig> = LazyLock::new(|| StorageConfig::init_from_env().unwrap());

#[derive(Envconfig)]
pub(crate) struct DatabaseConfig {
    #[envconfig(from = "DATABASE_MAX_CONNECTIONS", default = "5")]
    pub max_connections: u32,
    #[envconfig(
        from = "DATABASE_URL",
        default = "postgres://mango3:mango3@127.0.0.1:5432/boards_dev"
    )]
    pub url: String,
}

#[derive(Envconfig)]
pub(crate) struct MonitorConfig {
    #[envconfig(from = "MONITOR_REDIS_URL", default = "redis://127.0.0.1:6379/1")]
    pub redis_url: String,
}

#[derive(Envconfig)]
pub(crate) struct StorageConfig {
    #[envconfig(from = "STORAGE_PATH", default = "./storage/")]
    pub path: PathBuf,
    #[allow(dead_code)]
    #[envconfig(from = "STORAGE_URL", default = "http://127.0.0.1:8005/")]
    pub url: Url,
}
