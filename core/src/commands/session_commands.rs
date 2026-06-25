#[cfg(feature = "graphql")]
use std::net::IpAddr;

use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use uuid::Uuid;

#[cfg(feature = "graphql")]
use chrono::Utc;
#[cfg(feature = "graphql")]
use validator::Validate;

use crate::constants::{CACHE_PREFIX_GET_SESSION_BY_ID, CACHE_PREFIX_GET_SESSION_BY_TOKEN};
use crate::db_pool;
use crate::enums::CountryCode;
use crate::models::Session;

#[cfg(feature = "graphql")]
use crate::config::SESSION_CONFIG;
#[cfg(feature = "graphql")]
use crate::jobs_storage;
#[cfg(feature = "graphql")]
use crate::models::Application;
#[cfg(feature = "graphql")]
use crate::params::SessionParams;

use super::*;

#[cfg(feature = "graphql")]
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

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Session>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_SESSION_BY_ID).await }"##
)]
pub async fn get_session_by_id<'a>(id: Uuid) -> sqlx::Result<Session<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Session,
        r#"SELECT
            id,
            application_id,
            user_id,
            token,
            ip_address,
            country_code AS "country_code: CountryCode",
            region,
            city,
            expires_at,
            refreshed_at,
            finished_at,
            created_at,
            updated_at
        FROM sessions WHERE expires_at > current_timestamp AND finished_at IS NULL AND id = $1 LIMIT 1"#,
        id
    )
    .fetch_one(db_pool)
    .await
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ token.to_string() }"#,
    ty = "AsyncRedisCache<String, Session>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_SESSION_BY_TOKEN).await }"##
)]
pub async fn get_session_by_token<'a>(token: &str) -> sqlx::Result<Session<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Session,
        r#"SELECT
            id,
            application_id,
            user_id,
            token,
            ip_address,
            country_code AS "country_code: CountryCode",
            region,
            city,
            expires_at,
            refreshed_at,
            finished_at,
            created_at,
            updated_at
        FROM sessions WHERE token = $1 AND expires_at > current_timestamp AND finished_at IS NULL LIMIT 1"#,
        token
    )
    .fetch_one(db_pool)
    .await
}

#[cfg(feature = "graphql")]
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
        r#"INSERT INTO sessions (application_id, user_id, token, ip_address, expires_at) VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            application_id,
            user_id,
            token,
            ip_address,
            country_code AS "country_code: CountryCode",
            region,
            city,
            expires_at,
            refreshed_at,
            finished_at,
            created_at,
            updated_at"#,
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

#[cfg(feature = "graphql")]
pub(crate) async fn refresh_session<'a>(session: &Session<'_>) -> sqlx::Result<Session<'a>> {
    let db_pool = db_pool().await;

    let token = random_string(SESSION_CONFIG.token_length());
    let expires_at = Utc::now() + SESSION_CONFIG.ttl();

    let session = sqlx::query_as!(
        Session,
        r#"UPDATE sessions SET token = $2, expires_at = $3, refreshed_at = current_timestamp
        WHERE
            id = $1 AND finished_at IS NULL AND expires_at > current_timestamp
            AND (refreshed_at IS NULL OR refreshed_at < current_timestamp - INTERVAL '1 minute')
        RETURNING
            id,
            application_id,
            user_id,
            token,
            ip_address,
            country_code AS "country_code: CountryCode",
            region,
            city,
            expires_at,
            refreshed_at,
            finished_at,
            created_at,
            updated_at"#,
        session.id, // $1
        token,      // $2
        expires_at, // $3
    )
    .fetch_one(db_pool)
    .await?;

    remove_session_cache(&session).await;

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
    country_code: CountryCode,
    region: &str,
    city: &str,
) -> sqlx::Result<Session<'a>> {
    let db_pool = db_pool().await;

    let session = sqlx::query_as!(
        Session,
        r#"UPDATE sessions SET country_code = $2, region = $3, city = $4 WHERE finished_at IS NULL AND id = $1
        RETURNING
            id,
            application_id,
            user_id,
            token,
            ip_address,
            country_code AS "country_code: CountryCode",
            region,
            city,
            expires_at,
            refreshed_at,
            finished_at,
            created_at,
            updated_at"#,
        session.id,        // $1
        country_code as _, // $2
        region,            // $3
        city               // $4
    )
    .fetch_one(db_pool)
    .await?;

    remove_session_cache(&session).await;

    Ok(session)
}
