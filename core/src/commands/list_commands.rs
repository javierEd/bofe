use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use toolbox::cache::redis_cache_store;
use toolbox::constants::{ERROR_ALREADY_EXISTS, ERROR_IS_INVALID};
use toolbox::pagination::{CursorPage, CursorParams};
use toolbox::validator::{OrValidationErrors, ValidationResult};

use crate::constants::CACHE_PREFIX_GET_LIST_BY_ID;
use crate::db_pool;
use crate::models::{Board, List, User};
use crate::params::ListParams;

use super::get_board_by_id;

async fn list_name_exists(board: &Board<'_>, name: &str) -> bool {
    let db_pool = db_pool().await;

    sqlx::query!(
        "SELECT id FROM lists WHERE board_id = $1 AND LOWER(name) = $2 LIMIT 1",
        board.id,            // $1
        name.to_lowercase()  // $2
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, List<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_LIST_BY_ID).await }"##
)]
async fn get_list_by_id(id: Uuid) -> sqlx::Result<List<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        List,
        r#"SELECT * FROM lists WHERE id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_list<'a>(user: &User, params: ListParams) -> ValidationResult<List<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    let board = get_board_by_id(params.board_id).await.or_validation_errors()?;

    if board.user_id != user.id {
        validation_errors.add("board_id", ERROR_IS_INVALID.clone());

        return Err(validation_errors);
    }

    if list_name_exists(&board, &params.name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let position = suggest_list_position(&board).await;

    let db_pool = db_pool().await;

    sqlx::query_as!(
        List,
        "INSERT INTO lists (board_id, name, position) VALUES ($1, $2, $3) RETURNING *",
        board.id,    // $1
        params.name, // $2
        position,    // $3
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
