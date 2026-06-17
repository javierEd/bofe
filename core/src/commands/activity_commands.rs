use cached::AsyncRedisCache;
use cached::concurrent_cached;
use serde_json::Value;
use uuid::Uuid;

use crate::commands::notify_board_activities_channel;
use crate::constants::CACHE_PREFIX_GET_ACTIVITY_BY_ID;
use crate::db_pool;
use crate::enums::ActivityAction;
use crate::models::Board;
use crate::models::{Activity, User};
use crate::pagination::{CursorPage, CursorParams};

use super::{notify_board_channel, redis_cache_store};

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, Activity>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_ACTIVITY_BY_ID).await }"##
)]
pub(crate) async fn get_activity_by_id(id: Uuid) -> sqlx::Result<Activity> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Activity,
        r#"SELECT
            id,
            user_id,
            board_id,
            action AS "action!: ActivityAction",
            target_id,
            data,
            created_at
        FROM activities WHERE id = $1 LIMIT 1"#,
        id
    )
    .fetch_one(db_pool)
    .await
}

/// Get all activities in ascending order
pub async fn get_all_activities_by_board(board: &Board<'_>, after: Option<&Activity>) -> sqlx::Result<Vec<Activity>> {
    let db_pool = db_pool().await;
    let after_created_at = after.map(|a| a.created_at);

    sqlx::query_as!(
        Activity,
        r#"SELECT
            id,
            user_id,
            board_id,
            action AS "action!: ActivityAction",
            target_id,
            data,
            created_at
        FROM activities WHERE board_id = $1 AND ($2::timestamptz IS NULL OR created_at > $2) ORDER BY created_at ASC"#,
        board.id,         // $1
        after_created_at, // $2
    )
    .fetch_all(db_pool)
    .await
}

pub(crate) async fn get_last_activity_by_board(board: &Board<'_>) -> sqlx::Result<Activity> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Activity,
        r#"SELECT
            id,
            user_id,
            board_id,
            action AS "action!: ActivityAction",
            target_id,
            data,
            created_at
        FROM activities WHERE board_id = $1 ORDER BY created_at DESC LIMIT 1"#,
        board.id
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_activity(
    user: &User<'_>,
    board: &Board<'_>,
    action: ActivityAction,
    target_id: Uuid,
    data: &Value,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "INSERT INTO activities (user_id, board_id, action, target_id, data) VALUES ($1, $2, $3, $4, $5)",
        user.id,     // $1
        board.id,    // $2
        action as _, // $3
        target_id,   // $4
        data         // $5
    )
    .execute(db_pool)
    .await?;

    let _ = notify_board_channel(board).await;
    let _ = notify_board_activities_channel(board).await;

    Ok(())
}

pub(crate) async fn paginate_activities(
    cursor_params: CursorParams,
    user: Option<&User<'_>>,
    board: Option<&Board<'_>>,
    target_user: Option<&User<'_>>,
) -> CursorPage<Activity> {
    let db_pool = db_pool().await;
    let user_id = user.map(|u| u.id);
    let board_id = board.map(|b| b.id);
    let target_user_id = target_user.map(|u| u.id);

    CursorPage::new(
        &cursor_params,
        |node: &Activity| node.id,
        async |after| get_activity_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let cursor_created_at = cursor_resource.map(|c| c.created_at);

            sqlx::query_as!(
                Activity,
                r#"SELECT
                    a.id,
                    a.user_id,
                    a.board_id,
                    a.target_id,
                    a.action AS "action!: ActivityAction",
                    a.data,
                    a.created_at
                FROM activities AS a INNER JOIN boards AS b ON b.id = a.board_id
                WHERE
                    ($1::timestamptz IS NULL OR b.created_at < $1)
                    AND ($2::uuid IS NULL OR a.user_id = $2)
                    AND ($3::uuid IS NULL OR a.board_id = $3)
                    AND (
                        CASE b.visibility
                        WHEN 'public' THEN TRUE
                        WHEN 'users' THEN $4::uuid IS NOT NULL
                        ELSE
                            ($2 IS NOT NULL AND $2 = $4)
                            OR b.user_id = $4
                            OR (SELECT id FROM members WHERE board_id = b.id AND user_id = $4 LIMIT 1) IS NOT NULL
                        END
                    )
                ORDER BY a.created_at, a.target_id DESC LIMIT $5"#,
                cursor_created_at, // $1
                user_id,           // $2
                board_id,          // $3
                target_user_id,    // $4
                limit,             // $5
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}
