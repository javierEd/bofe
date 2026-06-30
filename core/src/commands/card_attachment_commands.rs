use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use uuid::Uuid;

use crate::constants::CACHE_PREFIX_GET_ALL_CARD_ATTACHMENTS;
use crate::db_pool;
use crate::models::{Attachment, Card, CardAttachment};

use super::{AsyncRedisCacheExt, redis_cache_store};

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ card.id }"#,
    ty = "AsyncRedisCache<Uuid, Vec<CardAttachment>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_ALL_CARD_ATTACHMENTS).await }"##
)]
pub async fn get_all_card_attachments(card: &Card<'_>) -> sqlx::Result<Vec<CardAttachment>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        CardAttachment,
        "SELECT * FROM card_attachments WHERE card_id = $1 ORDER BY created_at ASC",
        card.id, // $1
    )
    .fetch_all(db_pool)
    .await
}

pub async fn get_card_attachment(card_id: Uuid, attachment_id: Uuid) -> sqlx::Result<CardAttachment> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        CardAttachment,
        "SELECT * FROM card_attachments WHERE card_id = $1 AND attachment_id = $2 LIMIT 1",
        card_id,       // $1
        attachment_id  // $2
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_card_attachments_count(card: &Card<'_>) -> sqlx::Result<i64> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "SELECT COUNT(*) FROM card_attachments WHERE card_id = $1 LIMIT 1",
        card.id, // $1
    )
    .fetch_one(db_pool)
    .await
    .map(|record| record.count.unwrap_or_default())
}

pub async fn insert_card_attachments(card: &Card<'_>, attachments: &[Attachment<'_>]) -> sqlx::Result<()> {
    if attachments.is_empty() {
        return Ok(());
    }

    let db_pool = db_pool().await;

    let attachment_ids: Vec<Uuid> = attachments
        .iter()
        .filter_map(|attachment| {
            if attachment.user_id == card.user_id {
                Some(attachment.id)
            } else {
                None
            }
        })
        .collect();

    sqlx::query!(
        "INSERT INTO card_attachments (card_id, attachment_id) SELECT $1, UNNEST($2::uuid[])
        ON CONFLICT (card_id, attachment_id) DO NOTHING",
        card.id,         // $1
        &attachment_ids  // $2
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

async fn remove_all_card_attachments_cache(card: &Card<'_>) {
    GET_ALL_CARD_ATTACHMENTS
        .cache_remove(CACHE_PREFIX_GET_ALL_CARD_ATTACHMENTS, &card.id)
        .await;
}

pub async fn update_card_attachments(card: &Card<'_>, attachments: &[Attachment<'_>]) -> sqlx::Result<()> {
    insert_card_attachments(card, attachments).await?;

    let db_pool = db_pool().await;

    let attachment_ids: Vec<Uuid> = attachments.iter().map(|attachment| attachment.id).collect();

    sqlx::query!(
        "DELETE FROM card_attachments WHERE card_id = $1 AND attachment_id != ALL($2)",
        card.id,         // $1
        &attachment_ids  // $2
    )
    .execute(db_pool)
    .await?;

    remove_all_card_attachments_cache(card).await;

    Ok(())
}
