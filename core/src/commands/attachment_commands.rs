use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use uuid::Uuid;

use crate::constants::CACHE_PREFIX_GET_ATTACHMENT_BY_ID;
use crate::db_pool;
use crate::models::{Attachment, Blob, User};

use super::redis_cache_store;

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Attachment<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_ATTACHMENT_BY_ID).await }"##
)]
pub async fn get_attachment_by_id<'a>(id: Uuid) -> sqlx::Result<Attachment<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Attachment,
        "SELECT * FROM attachments WHERE id = $1 LIMIT 1",
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_attachments_by_ids<'a>(ids: &[Uuid]) -> sqlx::Result<Vec<Attachment<'a>>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Attachment,
        "SELECT * FROM attachments WHERE id = ANY($1)",
        ids, // $1
    )
    .fetch_all(db_pool)
    .await
}

pub async fn get_or_insert_attachment<'a>(
    user: &User<'_>,
    blob: &Blob<'_>,
    file_name: &str,
) -> sqlx::Result<Attachment<'a>> {
    let db_pool = db_pool().await;

    let file_name = file_name.trim();

    if let Ok(attachment) = sqlx::query_as!(
        Attachment,
        "SELECT * FROM attachments WHERE user_id = $1 AND blob_id = $2 AND LOWER(file_name) = $3",
        user.id,
        blob.id,
        file_name.to_lowercase(),
    )
    .fetch_one(db_pool)
    .await
    {
        return Ok(attachment);
    }

    sqlx::query_as!(
        Attachment,
        "INSERT INTO attachments (user_id, blob_id, file_name) VALUES ($1, $2, $3) RETURNING *",
        user.id,
        blob.id,
        file_name,
    )
    .fetch_one(db_pool)
    .await
}
