use cached::AsyncRedisCache;
use cached::concurrent_cached;
use uuid::Uuid;
use validator::ValidationErrors;

use crate::constants::{CACHE_PREFIX_GET_MEMBER, CACHE_PREFIX_GET_MEMBER_BY_ID, ERROR_IS_INVALID};
use crate::db_pool;
use crate::models::{Board, Member, User};
use crate::pagination::CursorPage;
use crate::pagination::CursorParams;
use crate::params::MemberParams;
use crate::params::UpdateMemberParams;

use super::*;

pub async fn delete_member(user: &User<'_>, member: &Member) -> sqlx::Result<bool> {
    if !member.is_removable(user).await? {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query!("DELETE FROM members WHERE id = $1", member.id)
        .execute(db_pool)
        .await?;

    remove_member_cache(member).await;

    Ok(true)
}

pub async fn get_admin_member(board: &Board<'_>, user: &User<'_>) -> sqlx::Result<Member> {
    let member = get_member(board, user).await?;

    if member.is_admin {
        Ok(member)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ format!("{}:{}", board.id, user.id) }"#,
    ty = "AsyncRedisCache<String, Member>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_MEMBER).await }"##
)]
pub async fn get_member(board: &Board<'_>, user: &User<'_>) -> sqlx::Result<Member> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Member,
        "SELECT * FROM members WHERE board_id = $1 AND user_id = $2 LIMIT 1",
        board.id, // $1
        user.id,  // $2
    )
    .fetch_one(db_pool)
    .await
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Member>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_MEMBER_BY_ID).await }"##
)]
pub async fn get_member_by_id(id: Uuid) -> sqlx::Result<Member> {
    let db_pool = db_pool().await;

    sqlx::query_as!(Member, "SELECT * FROM members WHERE id = $1 LIMIT 1", id,)
        .fetch_one(db_pool)
        .await
}
pub async fn insert_member(user: &User<'_>, params: MemberParams) -> ValidationResult<Member> {
    let board = get_visible_board_by_id(params.board_id, Some(user))
        .await
        .or_validation_errors()?;
    let target_user = get_user_by_id(params.user_id).await.or_validation_errors()?;

    let mut validation_errors = ValidationErrors::new();

    if !board.can_create_member(user) {
        validation_errors.add("board_id", ERROR_IS_INVALID.clone());

        return Err(validation_errors);
    }

    if target_user.id == user.id || target_user.id == board.user_id || get_member(&board, &target_user).await.is_ok() {
        validation_errors.add("user_id", ERROR_IS_INVALID.clone());

        return Err(validation_errors);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        Member,
        "INSERT INTO members (board_id, user_id, is_admin) VALUES ($1, $2, $3) RETURNING *",
        board.id,        // $1
        target_user.id,  // $2
        params.is_admin, // $3
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()
}

pub async fn paginate_members(cursor_params: CursorParams, board: &Board<'_>) -> CursorPage<Member> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &Member| node.id,
        async |after| get_member_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let cursor_created_at = cursor_resource.map(|c| c.created_at);

            sqlx::query_as!(
                Member,
                "SELECT * FROM members WHERE ($1::timestamptz IS NULL OR created_at < $1) AND board_id = $2
                ORDER BY created_at DESC
                LIMIT $3",
                cursor_created_at, // $1
                board.id,          // $2
                limit,             // $3
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}

async fn remove_member_cache(member: &Member) {
    GET_MEMBER
        .cache_remove(
            CACHE_PREFIX_GET_MEMBER,
            &format!("{}:{}", member.board_id, member.user_id),
        )
        .await;
}

pub async fn update_member(user: &User<'_>, member: &Member, params: UpdateMemberParams) -> ValidationResult<Member> {
    if !member.is_editable(user).await.or_validation_errors()? {
        return Err(ValidationErrors::new());
    }

    let db_pool = db_pool().await;

    let member = sqlx::query_as!(
        Member,
        "UPDATE members SET is_admin = $1 WHERE id = $2 RETURNING *",
        params.is_admin, // $1
        member.id,       // $2
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    remove_member_cache(&member).await;

    Ok(member)
}
