use crate::db_pool;
use crate::models::{Attachment, Blob, User};

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
