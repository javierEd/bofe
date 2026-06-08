use uuid::Uuid;

use crate::db_pool;
use crate::models::{Card, CardLabel, Label};

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

    Ok(())
}
