use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;

use uuid::Uuid;

use crate::constants::CACHE_PREFIX_GET_ALL_CARD_LABELS;
use crate::db_pool;
use crate::models::{Card, CardLabel};

#[cfg(feature = "graphql")]
use crate::models::Label;

#[cfg(feature = "graphql")]
use super::AsyncRedisCacheExt;

use super::redis_cache_store;

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ card.id }"#,
    ty = "AsyncRedisCache<Uuid, Vec<CardLabel>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_ALL_CARD_LABELS).await }"##
)]
pub async fn get_all_card_labels(card: &Card<'_>) -> sqlx::Result<Vec<CardLabel>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        CardLabel,
        "SELECT * FROM card_labels WHERE card_id = $1 ORDER BY created_at ASC",
        card.id, // $1
    )
    .fetch_all(db_pool)
    .await
}

#[cfg(feature = "graphql")]
pub async fn insert_card_labels(card: &Card<'_>, labels: &[Label<'_>]) -> sqlx::Result<()> {
    if labels.is_empty() {
        return Ok(());
    }

    let db_pool = db_pool().await;

    let board = card.board().await?;

    let label_ids: Vec<Uuid> = labels
        .iter()
        .filter_map(|label| {
            if board.id == label.board_id {
                Some(label.id)
            } else {
                None
            }
        })
        .collect();

    sqlx::query!(
        "INSERT INTO card_labels (card_id, label_id) SELECT $1, UNNEST($2::uuid[])
        ON CONFLICT (card_id, label_id) DO NOTHING",
        card.id,    // $1
        &label_ids  // $2
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[cfg(feature = "graphql")]
async fn remove_all_card_labels_cache(card: &Card<'_>) {
    GET_ALL_CARD_LABELS
        .cache_remove(CACHE_PREFIX_GET_ALL_CARD_LABELS, &card.id)
        .await;
}

#[cfg(feature = "graphql")]
pub async fn update_card_labels(card: &Card<'_>, labels: &[Label<'_>]) -> sqlx::Result<()> {
    insert_card_labels(card, labels).await?;

    let db_pool = db_pool().await;

    let label_ids: Vec<Uuid> = labels.iter().map(|label| label.id).collect();

    sqlx::query!(
        "DELETE FROM card_labels WHERE card_id = $1 AND label_id != ALL($2)",
        card.id,    // $1
        &label_ids  // $2
    )
    .execute(db_pool)
    .await?;

    remove_all_card_labels_cache(card).await;

    Ok(())
}
