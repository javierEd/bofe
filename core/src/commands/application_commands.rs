use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use chrono::Utc;
use validator::Validate;

use toolbox::cache::redis_cache_store;

use crate::config::APPLICATION_CONFIG;
use crate::constants::CACHE_PREFIX_GET_APPLICATION_BY_TOKEN;
use crate::db_pool;
use crate::models::Application;
use crate::params::ApplicationParams;

use super::{OrValidationErrors, ValidationResult, random_string};

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ token.to_string() }"#,
    ty = "AsyncRedisCache<String, Application<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_APPLICATION_BY_TOKEN).await }"##
)]
pub async fn get_application_by_token(token: &str) -> sqlx::Result<Application<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Application,
        "SELECT * FROM applications WHERE token = $1 AND expires_at > current_timestamp AND disabled_at IS NULL
        LIMIT 1",
        token
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_application<'a>(params: ApplicationParams) -> ValidationResult<Application<'a>> {
    params.validate()?;

    let db_pool = db_pool().await;

    let name = params.name.trim();
    let token = random_string(APPLICATION_CONFIG.token_length());
    let expires_at = params
        .expires_at
        .map(|date| date.and_time(Utc::now().time()).and_utc())
        .unwrap_or_else(|| Utc::now() + APPLICATION_CONFIG.ttl());

    let access_token = sqlx::query_as!(
        Application,
        "INSERT INTO applications (name, token, expires_at) VALUES ($1, $2, $3) RETURNING *",
        name,       // $1
        token,      // $2
        expires_at, // $3
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    Ok(access_token)
}
