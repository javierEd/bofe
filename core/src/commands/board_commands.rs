use cached::AsyncRedisCache;
use cached::proc_macro::io_cached;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use toolbox::cache::redis_cache_store;
use toolbox::constants::ERROR_ALREADY_EXISTS;
use toolbox::pagination::{CursorPage, CursorParams};
use toolbox::validator::{OrValidationErrors, ValidationResult};

use crate::constants::{CACHE_PREFIX_GET_BOARD_BY_ID, CACHE_PREFIX_GET_BOARD_BY_SLUG};
use crate::db_pool;
use crate::enums::BoardVisibility;
use crate::models::{Board, User};
use crate::params::BoardParams;

async fn board_name_exists(user: &User, name: &str) -> bool {
    let db_pool = db_pool().await;

    sqlx::query!(
        "SELECT id FROM boards WHERE user_id = $1 AND LOWER(name) = $2 LIMIT 1",
        user.id,             // $1
        name.to_lowercase()  // $2
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

async fn board_slug_exists(slug: &str) -> bool {
    let db_pool = db_pool().await;

    sqlx::query!(
        "SELECT id FROM boards WHERE LOWER(slug) = $1 LIMIT 1",
        slug.to_lowercase() // $2
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
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

pub async fn get_board_by_id_or_slug<'a>(id_or_slug: &str, target_user: Option<&User>) -> sqlx::Result<Board<'a>> {
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

pub async fn insert_board<'a>(user: &User, params: BoardParams) -> ValidationResult<Board<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    if board_name_exists(user, &params.name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if board_slug_exists(&params.slug).await {
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
        params.name,            // $2
        params.slug,            // $3
        params.description,     // $4
        params.visibility as _, // $5
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()
}

pub async fn paginate_boards<'a>(
    cursor_params: CursorParams,
    owner_user: Option<&User>,
    target_user: Option<&User>,
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
