use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use uuid::Uuid;

use toolbox::cache::redis_cache_store;
use toolbox::validator::{OrValidationErrors, ValidationResult};
use validator::Validate;

use crate::constants::*;
use crate::models::User;
use crate::params::UserParams;
use crate::{db_pool, jobs_storage};

use super::encrypt_password;

pub(crate) async fn authenticate_user<'a>(username_or_email: &str, password: &str) -> sqlx::Result<User<'a>> {
    let user = get_user_by_username_or_email(username_or_email).await?;

    if user.verify_password(password) {
        Ok(user)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, User>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_BY_ID).await }"##
)]
pub async fn get_user_by_id(id: Uuid) -> sqlx::Result<User<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE disabled_at IS NULL AND id = $1 LIMIT 1",
        id
    )
    .fetch_one(db_pool)
    .await
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ username.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, User<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_BY_USERNAME).await }"##
)]
pub(crate) async fn get_user_by_username(username: &str) -> sqlx::Result<User<'static>> {
    if username.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE disabled_at IS NULL AND LOWER(username) = LOWER($1) LIMIT 1",
        username
    )
    .fetch_one(db_pool)
    .await
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ username_or_email.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, User<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_BY_USERNAME_OR_EMAIL).await }"##
)]
async fn get_user_by_username_or_email(username_or_email: &str) -> sqlx::Result<User<'static>> {
    if username_or_email.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        "SELECT * FROM users
        WHERE disabled_at IS NULL AND (LOWER(username) = LOWER($1) OR LOWER(email) = LOWER($1))
        LIMIT 1",
        username_or_email
    )
    .fetch_one(db_pool)
    .await
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ email.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, Uuid>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_ID_BY_EMAIL).await }"##
)]
async fn get_user_id_by_email(email: &str) -> sqlx::Result<Uuid> {
    if email.is_empty() {
        return Err(sqlx::Error::InvalidArgument("email".to_owned()));
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        r#"SELECT id AS "id!" FROM users WHERE LOWER(email) = LOWER($1) LIMIT 1"#,
        email // $1
    )
    .fetch_one(db_pool)
    .await
    .map(|record| record.id)
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ username.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, Uuid>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_ID_BY_USERNAME).await }"##
)]
async fn get_user_id_by_username(username: &str) -> sqlx::Result<Uuid> {
    if username.is_empty() {
        return Err(sqlx::Error::InvalidArgument("username".to_owned()));
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        r#"SELECT id AS "id!" FROM users WHERE LOWER(username) = LOWER($1) LIMIT 1"#,
        username // $1
    )
    .fetch_one(db_pool)
    .await
    .map(|record| record.id)
}

pub(crate) async fn insert_user<'a>(params: UserParams) -> ValidationResult<User<'a>> {
    params.validate()?;

    let db_pool = db_pool().await;
    let display_name = params.full_name.split(' ').next().unwrap();
    let encrypted_password = encrypt_password(&params.password);

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (
            username,
            email,
            encrypted_password,
            display_name,
            full_name,
            birthdate,
            country_code
        ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        params.username,             // $1
        params.email.to_lowercase(), // $2
        encrypted_password,          // $3
        display_name,                // $4
        params.full_name,            // $5
        params.birthdate,            // $6
        params.country_code,         // $7
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    jobs_storage().await.push_new_user(&user).await;

    Ok(user)
}

pub(crate) async fn user_email_exists(email: &str) -> bool {
    get_user_id_by_email(email).await.is_ok()
}

pub(crate) async fn user_username_exists(username: &str) -> bool {
    get_user_id_by_username(username).await.is_ok()
}
