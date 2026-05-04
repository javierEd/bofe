use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use uuid::Uuid;

use toolbox::cache::redis_cache_store;
use toolbox::identity_client::{IdentityClient, IdentityUser};

use crate::constants::*;
use crate::models::User;
use crate::{db_pool, jobs_storage};

pub async fn get_user_by_identity_user(identity_user: &IdentityUser<'_>) -> sqlx::Result<User> {
    get_user_by_identity_user_id(identity_user.id).await
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, User>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_BY_IDENTITY_USER_ID).await }"##
)]
async fn get_user_by_identity_user_id(identity_user_id: Uuid) -> sqlx::Result<User> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE disabled_at IS NULL AND identity_user_id = $1 LIMIT 1",
        identity_user_id
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_or_insert_user(identity_client: &IdentityClient) -> anyhow::Result<User> {
    let identity_user = identity_client.current_user().await?;

    if let Ok(user) = get_user_by_identity_user(&identity_user).await {
        return Ok(user);
    }

    let db_pool = db_pool().await;

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (identity_user_id) VALUES ($1) RETURNING *",
        identity_user.id, // $1
    )
    .fetch_one(db_pool)
    .await?;

    jobs_storage().await.push_new_user(identity_client, &user).await;

    Ok(user)
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, User>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_BY_ID).await }"##
)]
pub async fn get_user_by_id(id: Uuid) -> sqlx::Result<User> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE disabled_at IS NULL AND id = $1 LIMIT 1",
        id
    )
    .fetch_one(db_pool)
    .await
}
