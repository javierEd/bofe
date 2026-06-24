use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use uuid::Uuid;

use crate::constants::CACHE_PREFIX_GET_ATTACHMENT_KEY_BY_ID;
use crate::db_pool;
use crate::models::{Attachment, AttachmentKey, User};

use super::redis_cache_store;

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, AttachmentKey>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_ATTACHMENT_KEY_BY_ID).await }"##
)]
pub async fn get_attachment_key_by_id(id: Uuid) -> sqlx::Result<AttachmentKey> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        AttachmentKey,
        "SELECT * FROM attachment_keys WHERE id = $1 AND expires_at > current_timestamp LIMIT 1",
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub(crate) async fn insert_attachment_key(
    user: Option<&User<'_>>,
    attachment: &Attachment<'_>,
) -> sqlx::Result<AttachmentKey> {
    let db_pool = db_pool().await;
    let user_id = user.map(|u| u.id);

    sqlx::query_as!(
        AttachmentKey,
        "INSERT INTO attachment_keys (user_id, attachment_id) VALUES ($1, $2) RETURNING *",
        user_id,       // $1
        attachment.id, // $2
    )
    .fetch_one(db_pool)
    .await
}
