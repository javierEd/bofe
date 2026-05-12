use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::constants::{CACHE_PREFIX_GET_BOARD_BY_ID, CACHE_PREFIX_GET_BOARD_BY_SLUG, ERROR_ALREADY_EXISTS};
use crate::db_pool;
use crate::enums::BoardVisibility;
use crate::models::{Board, User};
use crate::pagination::{CursorPage, CursorParams};
use crate::params::BoardParams;

use super::{AsyncRedisCacheExt, OrValidationErrors, ValidationResult, redis_cache_store};

async fn board_name_exists(user: &User<'_>, board: Option<&Board<'_>>, name: &str) -> bool {
    let db_pool = db_pool().await;
    let board_id = board.map(|b| b.id);

    sqlx::query!(
        "SELECT id FROM boards WHERE user_id = $1 AND id != $2 AND LOWER(name) = LOWER($3) LIMIT 1",
        user.id,  // $1
        board_id, // $2
        name      // $3
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

async fn board_slug_exists(board: Option<&Board<'_>>, slug: &str) -> bool {
    let db_pool = db_pool().await;
    let board_id = board.map(|b| b.id);

    sqlx::query!(
        "SELECT id FROM boards WHERE id != $1 AND LOWER(slug) = LOWER($2) LIMIT 1",
        board_id, // $1
        slug      // $2
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

pub async fn delete_board(user: &User<'_>, board: &Board<'_>) -> sqlx::Result<bool> {
    if !board.is_editable(Some(user)) {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM boards WHERE id = $1", board.id)
        .execute(db_pool)
        .await?;

    remove_board_cache(board).await;

    Ok(true)
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Board<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_BOARD_BY_ID).await }"##
)]
pub async fn get_board_by_id(id: Uuid) -> sqlx::Result<Board<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Board,
        r#"SELECT
            id,
            user_id,
            name,
            slug,
            description,
            visibility as "visibility!: BoardVisibility",
            created_at,
            updated_at
        FROM boards WHERE id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_board_by_id_or_slug<'a>(id_or_slug: &str, target_user: Option<&User<'_>>) -> sqlx::Result<Board<'a>> {
    let board = if let Ok(id) = Uuid::try_parse(id_or_slug) {
        get_board_by_id(id).await
    } else {
        get_board_by_slug(id_or_slug).await
    }?;

    if board.is_visible(target_user) {
        Ok(board)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

#[io_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ slug.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, Board<'_>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_BOARD_BY_SLUG).await }"##
)]
async fn get_board_by_slug(slug: &str) -> sqlx::Result<Board<'static>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Board,
        r#"SELECT
            id,
            user_id,
            name,
            slug,
            description,
            visibility as "visibility!: BoardVisibility",
            created_at,
            updated_at
        FROM boards WHERE LOWER(slug) = $1 LIMIT 1"#,
        slug.to_lowercase(), // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_board<'a>(user: &User<'_>, params: BoardParams) -> ValidationResult<Board<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    let name = params.name.trim();
    let slug = params.slug.trim().to_lowercase();
    let description = params.description.trim();

    if board_name_exists(user, None, name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if board_slug_exists(None, &slug).await {
        validation_errors.add("slug", ERROR_ALREADY_EXISTS.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        Board,
        r#"INSERT INTO boards (user_id, name, slug, description, visibility) VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            user_id,
            name,
            slug,
            description,
            visibility AS "visibility!: BoardVisibility",
            created_at,
            updated_at"#,
        user.id,                // $1
        name,                   // $2
        slug,                   // $3
        description,            // $4
        params.visibility as _, // $5
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()
}

pub async fn paginate_boards<'a>(
    cursor_params: CursorParams,
    owner_user: Option<&User<'_>>,
    target_user: Option<&User<'_>>,
) -> CursorPage<Board<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &Board| node.id,
        async |after| get_board_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let cursor_name = cursor_resource.map(|c| c.name.to_string());
            let owner_user_id = owner_user.map(|u| u.id);
            let target_user_id = target_user.map(|u| u.id);

            sqlx::query_as!(
                Board,
                r#"SELECT
                    id,
                    user_id,
                    name,
                    slug,
                    description,
                    visibility as "visibility!: BoardVisibility",
                    created_at,
                    updated_at
                FROM boards
                WHERE
                    ($1::text IS NULL OR name > $1)
                    AND ($2::uuid IS NULL OR user_id = $2)
                    AND (user_id = $3 OR (visibility = 'users' AND $3 IS NOT NULL) OR visibility = 'public')
                ORDER BY name ASC LIMIT $4"#,
                cursor_name,    // $1
                owner_user_id,  // $2
                target_user_id, // $3
                limit,          // $4
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}

async fn remove_board_cache(board: &Board<'_>) {
    let slug = board.slug.to_lowercase();

    tokio::join!(
        GET_BOARD_BY_ID.cache_remove(CACHE_PREFIX_GET_BOARD_BY_ID, &board.id),
        GET_BOARD_BY_SLUG.cache_remove(CACHE_PREFIX_GET_BOARD_BY_SLUG, &slug),
    );
}

pub async fn update_board<'a>(user: &User<'_>, board: &Board<'_>, params: BoardParams) -> ValidationResult<Board<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    if !board.is_editable(Some(user)) {
        return Err(validation_errors);
    }

    let name = params.name.trim();
    let slug = params.slug.trim().to_lowercase();
    let description = params.description.trim();

    if board_name_exists(user, Some(board), name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if board_slug_exists(Some(board), &slug).await {
        validation_errors.add("slug", ERROR_ALREADY_EXISTS.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let db_pool = db_pool().await;

    let board = sqlx::query_as!(
        Board,
        r#"UPDATE boards SET name = $2, slug = $3, description = $4, visibility = $5 WHERE id = $1
        RETURNING
            id,
            user_id,
            name,
            slug,
            description,
            visibility AS "visibility!: BoardVisibility",
            created_at,
            updated_at"#,
        board.id,               // $1
        name,                   // $2
        slug,                   // $3
        description,            // $4
        params.visibility as _, // $5
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    remove_board_cache(&board).await;

    Ok(board)
}
