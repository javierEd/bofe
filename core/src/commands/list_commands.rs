use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use toolbox::constants::{ERROR_ALREADY_EXISTS, ERROR_IS_INVALID};
use toolbox::pagination::{CursorPage, CursorParams};
use toolbox::validator::{OrValidationErrors, ValidationResult};

use crate::db_pool;
use crate::models::{Board, List, User};
use crate::params::{ListParams, UpdateListParams};

use super::get_board_by_id;

pub(crate) async fn delete_list(user: &User<'_>, list: &List<'_>) -> sqlx::Result<bool> {
    if !list.is_editable(Some(user)) {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        "DELETE FROM lists WHERE id = $1",
        list.id, // $1
    )
    .execute(db_pool)
    .await?;

    Ok(true)
}

async fn list_name_exists(board: &Board<'_>, list: Option<&List<'_>>, name: &str) -> bool {
    let db_pool = db_pool().await;
    let list_id = list.map(|l| l.id);

    sqlx::query!(
        "SELECT id FROM lists WHERE board_id = $1 AND id != $2 AND LOWER(name) = $3 LIMIT 1",
        board.id,            // $1
        list_id,             // $2
        name.to_lowercase()  // $3
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

pub async fn get_list_by_id<'a>(id: Uuid) -> sqlx::Result<List<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        List,
        r#"SELECT * FROM lists WHERE id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_list<'a>(user: &User<'_>, params: ListParams) -> ValidationResult<List<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    let board = get_board_by_id(params.board_id).await.or_validation_errors()?;

    if !board.is_editable(Some(user)) {
        validation_errors.add("board_id", ERROR_IS_INVALID.clone());

        return Err(validation_errors);
    }

    let name = params.name.trim();

    if list_name_exists(&board, None, name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let position = suggest_list_position(&board).await;

    let db_pool = db_pool().await;

    sqlx::query_as!(
        List,
        "INSERT INTO lists (board_id, user_id, name, position) VALUES ($1, $2, $3, $4) RETURNING *",
        board.id, // $1
        user.id,  // $2
        name,     // $3
        position, // $4
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()
}

async fn suggest_list_position(board: &Board<'_>) -> i16 {
    let db_pool = db_pool().await;

    sqlx::query_scalar!(
        r#"SELECT MAX(position) as "max_position!" FROM lists WHERE board_id = $1 LIMIT 1"#,
        board.id
    )
    .fetch_one(db_pool)
    .await
    .map(|max| max + 1)
    .unwrap_or(0)
}

pub async fn paginate_lists<'a>(cursor_params: CursorParams, board: &Board<'a>) -> CursorPage<List<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &List| node.id,
        async |after| get_list_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let cursor_position = cursor_resource.map(|c| c.position);

            sqlx::query_as!(
                List,
                "SELECT * FROM lists WHERE ($1::smallint IS NULL OR position > $1) AND board_id = $2
                ORDER BY position ASC
                LIMIT $3",
                cursor_position, // $1
                board.id,        // $2
                limit,           // $3
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}

pub async fn update_list<'a>(user: &User<'_>, list: &List<'_>, params: UpdateListParams) -> ValidationResult<List<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    if !list.is_editable(Some(user)) {
        return Err(validation_errors);
    }

    let name = params.name.trim();

    let board = list.board().await.or_validation_errors()?;

    if list_name_exists(&board, Some(list), name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        List,
        "UPDATE lists SET name = $2 WHERE id = $1 RETURNING *",
        list.id, // $1
        name,    // $2
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()
}

pub async fn update_list_position<'a>(user: &User<'_>, list: &List<'_>, position: i16) -> ValidationResult<List<'a>> {
    let board = list.board().await.or_validation_errors()?;

    if !board.is_editable(Some(user)) || position < 0 || position == list.position {
        return Err(ValidationErrors::new());
    }

    let mut transaction = db_pool().await.begin().await.or_validation_errors()?;

    sqlx::query!("SET CONSTRAINTS ALL DEFERRED")
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;

    sqlx::query!("UPDATE lists SET position = -1 WHERE id = $1", list.id)
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;

    if position > list.position {
        sqlx::query!(
            "UPDATE lists SET position = position - 1 WHERE board_id = $1 AND position BETWEEN $2 AND $3",
            board.id,          // $1
            list.position + 1, // $2
            position,          // $3
        )
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;
    } else {
        sqlx::query!(
            "UPDATE lists SET position = position + 1 WHERE board_id = $1 AND position BETWEEN $2 AND $3",
            board.id,          // $1
            position,          // $2
            list.position - 1, // $3
        )
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;
    }

    let list = sqlx::query_as!(
        List,
        "UPDATE lists SET position = $1 WHERE id = $2 RETURNING *",
        position, // $1
        list.id,  // $2
    )
    .fetch_one(&mut *transaction)
    .await
    .or_validation_errors()?;

    transaction.commit().await.or_validation_errors()?;

    Ok(list)
}
