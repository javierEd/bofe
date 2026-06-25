use std::fmt::Display;

use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use uuid::Uuid;

#[cfg(feature = "graphql")]
use validator::{Validate, ValidationErrors};

use crate::constants::*;
use crate::db_pool;
use crate::enums::BoardVisibility;
use crate::models::{Board, User};

#[cfg(feature = "graphql")]
use crate::enums::ActivityAction;
#[cfg(feature = "graphql")]
use crate::jobs_storage;
#[cfg(feature = "graphql")]
use crate::pagination::{CursorPage, CursorParams};
#[cfg(feature = "graphql")]
use crate::params::BoardParams;

use super::redis_cache_store;

#[cfg(feature = "graphql")]
use super::{AsyncRedisCacheExt, OrValidationErrors, ValidationResult, notify_board_channel};

#[derive(Clone)]
struct UuidAndString(Uuid, String);

impl Display for UuidAndString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[cfg(feature = "graphql")]
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

#[cfg(feature = "graphql")]
async fn board_slug_exists(user: &User<'_>, board: Option<&Board<'_>>, slug: &str) -> bool {
    let db_pool = db_pool().await;
    let board_id = board.map(|b| b.id);

    sqlx::query!(
        "SELECT id FROM boards WHERE user_id = $1 AND id != $2 AND LOWER(slug) = LOWER($3) LIMIT 1",
        user.id,  // $1
        board_id, // $2
        slug      // $3
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

#[cfg(feature = "graphql")]
pub(crate) async fn delete_board(user: &User<'_>, board: &Board<'_>) -> sqlx::Result<bool> {
    if !board.is_editable(user) {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM boards WHERE id = $1", board.id)
        .execute(db_pool)
        .await?;

    remove_board_cache(board).await;

    let _ = notify_board_channel(board).await;

    Ok(true)
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Board>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_BOARD_BY_ID).await }"##
)]
pub async fn get_board_by_id<'a>(id: Uuid) -> sqlx::Result<Board<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Board,
        r#"SELECT
            id,
            user_id,
            name,
            slug,
            description,
            visibility AS "visibility!: BoardVisibility",
            created_at,
            updated_at
        FROM boards WHERE id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

// TODO: To be removed
#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ slug.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, Board>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_BOARD_BY_SLUG).await }"##
)]
async fn get_board_by_slug<'a>(slug: &str) -> sqlx::Result<Board<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Board,
        r#"SELECT
            id,
            user_id,
            name,
            slug,
            description,
            visibility AS "visibility!: BoardVisibility",
            created_at,
            updated_at
        FROM boards WHERE LOWER(slug) = $1 LIMIT 1"#,
        slug.to_lowercase(), // $1
    )
    .fetch_one(db_pool)
    .await
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ UuidAndString(user.id, slug.to_lowercase()) }"#,
    ty = "AsyncRedisCache<UuidAndString, Board>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_BOARD_BY_USER_AND_SLUG).await }"##
)]
pub(crate) async fn get_board_by_user_and_slug<'a>(user: &User<'_>, slug: &str) -> sqlx::Result<Board<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Board,
        r#"SELECT
            id,
            user_id,
            name,
            slug,
            description,
            visibility AS "visibility!: BoardVisibility",
            created_at,
            updated_at
        FROM boards WHERE user_id = $1 AND LOWER(slug) = $2 LIMIT 1"#,
        user.id,             // $1
        slug.to_lowercase(), // $2
    )
    .fetch_one(db_pool)
    .await
}

#[cfg(feature = "graphql")]
pub(crate) async fn get_visible_board_by_id<'a>(id: Uuid, target_user: Option<&User<'_>>) -> sqlx::Result<Board<'a>> {
    let board = get_board_by_id(id).await?;

    if board.is_visible(target_user).await {
        Ok(board)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

#[cfg(feature = "graphql")]
pub(crate) async fn get_visible_board_by_slug<'a>(
    slug: &str,
    target_user: Option<&User<'_>>,
) -> sqlx::Result<Board<'a>> {
    let board = get_board_by_slug(slug).await?;

    if board.is_visible(target_user).await {
        Ok(board)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

#[cfg(feature = "graphql")]
pub(crate) async fn get_visible_board_by_user_and_slug<'a>(
    user: &User<'_>,
    slug: &str,
    target_user: Option<&User<'_>>,
) -> sqlx::Result<Board<'a>> {
    let board = get_board_by_user_and_slug(user, slug).await?;

    if board.is_visible(target_user).await {
        Ok(board)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

#[cfg(feature = "graphql")]
pub(crate) async fn has_boards(user: &User<'_>) -> bool {
    let db_pool = db_pool().await;

    sqlx::query!(
        "SELECT id FROM boards WHERE user_id = $1 LIMIT 1",
        user.id, // $1
    )
    .fetch_one(db_pool)
    .await
    .is_ok()
}

#[cfg(feature = "graphql")]
pub(crate) async fn insert_board<'a>(user: &User<'_>, params: BoardParams) -> ValidationResult<Board<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    let name = params.name.trim();
    let slug = params.slug.trim().to_lowercase();
    let description = params.description.trim();

    if board_name_exists(user, None, name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if board_slug_exists(user, None, &slug).await {
        validation_errors.add("slug", ERROR_ALREADY_EXISTS.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let db_pool = db_pool().await;

    let board = sqlx::query_as!(
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
    .or_validation_errors()?;

    jobs_storage()
        .await
        .push_activity(user, &board, ActivityAction::CreateBoard, &board, &board)
        .await;

    Ok(board)
}

#[cfg(feature = "graphql")]
pub(crate) async fn paginate_boards<'a>(
    cursor_params: CursorParams,
    member_user: Option<&User<'_>>,
    target_user: Option<&User<'_>>,
) -> CursorPage<Board<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &Board| node.id,
        async |after| get_board_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let cursor_name = cursor_resource.map(|c| c.name.to_string());
            let member_user_id = member_user.map(|u| u.id);
            let target_user_id = target_user.map(|u| u.id);

            sqlx::query_as!(
                Board,
                r#"SELECT
                    id,
                    user_id,
                    name,
                    slug,
                    description,
                    visibility AS "visibility!: BoardVisibility",
                    created_at,
                    updated_at
                FROM boards AS b
                WHERE
                    ($1::text IS NULL OR name > $1)
                    AND (
                        $2::uuid IS NULL OR user_id = $2
                        OR (SELECT id FROM members WHERE board_id = b.id AND user_id = $2 LIMIT 1) IS NOT NULL
                    ) AND (
                        CASE visibility
                        WHEN 'public' THEN TRUE
                        WHEN 'users' THEN $3::uuid IS NOT NULL
                        ELSE
                            ($2 IS NOT NULL AND $2 = $3)
                            OR user_id = $3
                            OR (SELECT id FROM members WHERE board_id = b.id AND user_id = $3 LIMIT 1) IS NOT NULL
                        END
                    )
                ORDER BY name ASC LIMIT $4"#,
                cursor_name,    // $1
                member_user_id, // $2
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

#[cfg(feature = "graphql")]
async fn remove_board_cache(board: &Board<'_>) {
    let slug = board.slug.to_lowercase();
    let user_and_slug = UuidAndString(board.user_id, slug.clone());

    tokio::join!(
        GET_BOARD_BY_ID.cache_remove(CACHE_PREFIX_GET_BOARD_BY_ID, &board.id),
        GET_BOARD_BY_SLUG.cache_remove(CACHE_PREFIX_GET_BOARD_BY_SLUG, &slug),
        GET_BOARD_BY_USER_AND_SLUG.cache_remove(CACHE_PREFIX_GET_BOARD_BY_USER_AND_SLUG, &user_and_slug)
    );
}

#[cfg(feature = "graphql")]
pub(crate) async fn update_board<'a>(
    user: &User<'_>,
    board: &Board<'a>,
    params: BoardParams,
) -> ValidationResult<Board<'a>> {
    params.validate()?;

    if params.name == board.name
        && params.slug == board.slug
        && params.description == board.description
        && params.visibility == board.visibility
    {
        return Ok(board.clone());
    }

    let mut validation_errors = ValidationErrors::new();

    if !board.is_editable(user) {
        return Err(validation_errors);
    }

    let name = params.name.trim();
    let slug = params.slug.trim().to_lowercase();
    let description = params.description.trim();

    if board_name_exists(user, Some(board), name).await {
        validation_errors.add("name", ERROR_ALREADY_EXISTS.clone());
    }

    if board_slug_exists(user, Some(board), &slug).await {
        validation_errors.add("slug", ERROR_ALREADY_EXISTS.clone());
    }

    if !validation_errors.is_empty() {
        return Err(validation_errors);
    }

    let db_pool = db_pool().await;

    let updated_board = sqlx::query_as!(
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

    remove_board_cache(board).await;

    jobs_storage()
        .await
        .push_activity(
            user,
            &updated_board,
            ActivityAction::UpdateBoard,
            &updated_board,
            &updated_board,
        )
        .await;

    Ok(updated_board)
}
