use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::OnceCell;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

mod config;
mod constants;
mod pagination;
mod scalars;

pub mod commands;
pub mod enums;
pub mod jobs;
pub mod models;

#[cfg(feature = "graphql")]
pub mod graphql;
#[cfg(feature = "graphql")]
pub mod params;

use crate::config::{DATABASE_CONFIG, IM_DATABASE_CONFIG};
use crate::enums::LanguageCode;
use crate::jobs::JobsStorage;

static DB_POOL_CELL: OnceCell<PgPool> = OnceCell::const_new();
static JOBS_STORAGE_CELL: OnceCell<JobsStorage> = OnceCell::const_new();
static IM_DB_CLIENT: LazyLock<redis::Client> =
    LazyLock::new(|| redis::Client::open(IM_DATABASE_CONFIG.url.clone()).expect("Could not get Redis client"));

fluent_templates::static_loader! {
    static LOCALES = {
        locales: "../locales",
        fallback_language: "en"
    };
}

#[derive(Clone, Default)]
pub struct L10n(LanguageIdentifier);

impl From<&LanguageCode> for L10n {
    fn from(code: &LanguageCode) -> Self {
        Self(code.lang_id())
    }
}

impl From<&str> for L10n {
    fn from(lang: &str) -> Self {
        Self(LanguageCode::from(lang).lang_id())
    }
}

impl L10n {
    pub fn text(&self, text_id: &str) -> String {
        LOCALES.lookup(&self.0, text_id)
    }

    pub fn text_with_args(&self, text_id: &str, args: &HashMap<Cow<'static, str>, FluentValue>) -> String {
        LOCALES.lookup_with_args(&self.0, text_id, args)
    }
}

#[derive(Serialize)]
pub struct Info {
    pub built_at: DateTime<Utc>,
    pub version: String,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            built_at: env!("BUILD_DATETIME").parse().unwrap(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[cfg(feature = "graphql")]
fn block_on<T>(f: impl Future<Output = T>) -> T {
    tokio::task::block_in_place(move || tokio::runtime::Handle::current().block_on(f))
}

async fn db_pool<'a>() -> &'a PgPool {
    DB_POOL_CELL
        .get_or_init(|| async {
            PgPoolOptions::new()
                .max_connections(DATABASE_CONFIG.max_connections)
                .connect(&DATABASE_CONFIG.url)
                .await
                .expect("Could not create DB pool.")
        })
        .await
}

pub async fn jobs_storage<'a>() -> &'a JobsStorage {
    JOBS_STORAGE_CELL
        .get_or_init(|| async { JobsStorage::new().await })
        .await
}

/// In-memory Database client
fn im_db_client() -> redis::Client {
    IM_DB_CLIENT.clone()
}

pub fn start_tracing_subscriber() {
    let env_filter = EnvFilter::from_default_env();
    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(env_filter);

    tracing_subscriber::registry().with(fmt_layer).init();

    tracing::info!("Tracing subscriber initialized.");
}
