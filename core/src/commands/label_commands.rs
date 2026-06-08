use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::constants::{CACHE_PREFIX_GET_LABEL_BY_ID, ERROR_ALREADY_EXISTS, ERROR_IS_INVALID};
use crate::db_pool;
use crate::models::{Board, Label, User};
use crate::pagination::{CursorPage, CursorParams};
use crate::params::{LabelParams, UpdateLabelParams};

use super::*;

pub(crate) async fn delete_label(user: &User<'_>, label: &Label<'_>) -> sqlx::Result<bool> {
    if !label.is_editable(user).await? {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        "DELETE FROM labels WHERE id = $1",
        label.id, // $1
    )
    .execute(db_pool)
    .await?;

    remove_label_cache(label).await;

    let _ = notify_board_channel(&label.board().await?).await;

    Ok(true)
}

pub async fn get_all_labels<'a>(board: &Board<'a>) -> sqlx::Result<Vec<Label<'a>>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Label,
        "SELECT * FROM labels WHERE board_id = $1 ORDER BY name ASC",
        board.id, // $1
    )
    .fetch_all(db_pool)
    .await
}

async fn label_name_exists(board: &Board<'_>, label: Option<&Label<'_>>, name: &str) -> bool {
    let db_pool = db_pool().await;
    let label_id = label.map(|l| l.id);

    sqlx::query!(
        "SELECT id FROM labels WHERE board_id = $1 AND id != $2 AND LOWER(name) = $3 LIMIT 1",
        board.id,            // $1
        label_id,            // $2
        name.to_lowercase()  // $3
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Label<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_LABEL_BY_ID).await }"##
)]
pub async fn get_label_by_id<'a>(id: Uuid) -> sqlx::Result<Label<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Label,
        r#"SELECT * FROM labels WHERE id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_visible_label_by_id<'a>(id: Uuid, target_user: Option<&User<'_>>) -> sqlx::Result<Label<'a>> {
    let label = get_label_by_id(id).await?;

    if label.is_visible(target_user).await? {
        Ok(label)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

pub async fn get_visible_labels_by_ids<'a>(
    ids: &[Uuid],
    target_user: Option<&User<'_>>,
) -> sqlx::Result<Vec<Label<'a>>> {
    futures::future::try_join_all(ids.iter().map(|id| get_visible_label_by_id(*id, target_user))).await
}

pub async fn insert_label<'a>(user: &User<'_>, params: LabelParams) -> ValidationResult<Label<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    let board = get_visible_board_by_id(params.board_id, Some(user))
        .await
        .or_validation_errors()?;

    if !board.can_create_label(user) {
        validation_errors.add("board_id", ERROR_IS_INVALID.clone());

        return Err(validation_errors);
    }

    let name = params.name.trim();

    if label_name_exists(&board, None, name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let db_pool = db_pool().await;

    let label = sqlx::query_as!(
        Label,
        "INSERT INTO labels (board_id, user_id, name, color_code) VALUES ($1, $2, $3, $4) RETURNING *",
        board.id,               // $1
        user.id,                // $2
        name,                   // $3
        params.color_code as _, // $4
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    let _ = notify_board_channel(&board).await;

    Ok(label)
}

pub async fn paginate_labels<'a>(cursor_params: CursorParams, board: &Board<'a>) -> CursorPage<Label<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &Label| node.id,
        async |after| get_label_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let cursor_name = cursor_resource.map(|c| c.name.to_string());

            sqlx::query_as!(
                Label,
                "SELECT * FROM labels WHERE ($1::text IS NULL OR name > $1) AND board_id = $2
                ORDER BY name ASC
                LIMIT $3",
                cursor_name, // $1
                board.id,    // $2
                limit,       // $3
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}

async fn remove_label_cache(label: &Label<'_>) {
    GET_LABEL_BY_ID
        .cache_remove(CACHE_PREFIX_GET_LABEL_BY_ID, &label.id)
        .await;
}

pub async fn update_label<'a>(
    user: &User<'_>,
    label: &Label<'_>,
    params: UpdateLabelParams,
) -> ValidationResult<Label<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    if !label.is_editable(user).await.or_validation_errors()? {
        return Err(validation_errors);
    }

    let name = params.name.trim();

    let board = label.board().await.or_validation_errors()?;

    if label_name_exists(&board, Some(label), name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());

        return Err(validation_errors);
    }

    let db_pool = db_pool().await;

    let updated_label = sqlx::query_as!(
        Label,
        "UPDATE labels SET name = $2, color_code = $3 WHERE id = $1 RETURNING *",
        label.id,               // $1
        name,                   // $2
        params.color_code as _, // $3
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    remove_label_cache(label).await;

    let _ = notify_board_channel(&board).await;

    Ok(updated_label)
}
