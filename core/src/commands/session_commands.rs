use std::net::IpAddr;

use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use crate::config::SESSION_CONFIG;
use crate::constants::{CACHE_PREFIX_GET_SESSION_BY_ID, CACHE_PREFIX_GET_SESSION_BY_TOKEN};
use crate::models::{Application, Session};
use crate::params::SessionParams;
use crate::{db_pool, jobs_storage};

use super::*;

pub(crate) async fn finish_session(session: &Session<'_>) -> sqlx::Result<bool> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE sessions SET finished_at = current_timestamp
        WHERE id = $1 AND finished_at IS NULL AND expires_at > current_timestamp",
        session.id, // $1
    )
    .execute(db_pool)
    .await?;

    remove_session_cache(session).await;

    Ok(true)
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Session>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_SESSION_BY_ID).await }"##
)]
pub async fn get_session_by_id(id: Uuid) -> sqlx::Result<Session<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Session,
        "SELECT * FROM sessions WHERE expires_at > current_timestamp AND finished_at IS NULL AND id = $1 LIMIT 1",
        id
    )
    .fetch_one(db_pool)
    .await
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ token.to_string() }"#,
    ty = "AsyncRedisCache<String, Session>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_SESSION_BY_TOKEN).await }"##
)]
pub async fn get_session_by_token(token: &str) -> sqlx::Result<Session<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Session,
        "SELECT * FROM sessions WHERE token = $1 AND expires_at > current_timestamp AND finished_at IS NULL LIMIT 1",
        token
    )
    .fetch_one(db_pool)
    .await
}

pub(crate) async fn insert_session<'a>(
    application: &Application<'_>,
    ip_address: &IpAddr,
    params: SessionParams,
) -> ValidationResult<Session<'a>> {
    params.validate()?;

    let user = authenticate_user(&params.username_or_email, &params.password)
        .await
        .or_validation_errors()?;

    let db_pool = db_pool().await;

    let token = random_string(SESSION_CONFIG.token_length());
    let expires_at = Utc::now() + SESSION_CONFIG.ttl();

    let session = sqlx::query_as!(
        Session,
        "INSERT INTO sessions (application_id, user_id, token, ip_address, expires_at) VALUES ($1, $2, $3, $4, $5)
        RETURNING *",
        application.id,         // $1
        user.id,                // $2
        token,                  // $3
        ip_address.to_string(), // $4
        expires_at,             // $5
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    jobs_storage().await.push_new_session(&session).await;

    Ok(session)
}

async fn remove_session_cache(session: &Session<'_>) {
    let token = session.token.to_string();

    tokio::join!(
        GET_SESSION_BY_ID.cache_remove(CACHE_PREFIX_GET_SESSION_BY_ID, &session.id),
        GET_SESSION_BY_TOKEN.cache_remove(CACHE_PREFIX_GET_SESSION_BY_TOKEN, &token),
    );
}

pub async fn update_session_location<'a>(
    session: &Session<'_>,
    country_code: &str,
    region: &str,
    city: &str,
) -> sqlx::Result<Session<'a>> {
    let db_pool = db_pool().await;

    let session = sqlx::query_as!(
        Session,
        "UPDATE sessions SET country_code = $2, region = $3, city = $4 WHERE finished_at IS NULL AND id = $1
        RETURNING *",
        session.id,   // $1
        country_code, // $2
        region,       // $3
        city          // $4
    )
    .fetch_one(db_pool)
    .await?;

    remove_session_cache(&session).await;

    Ok(session)
}
