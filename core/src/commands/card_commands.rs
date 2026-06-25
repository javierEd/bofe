use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::constants::{CACHE_PREFIX_GET_ALL_CARDS, ERROR_IS_INVALID};
use crate::enums::ActivityAction;
use crate::models::{Card, List, User};
use crate::pagination::{CursorPage, CursorParams};
use crate::params::CardParams;
use crate::{db_pool, jobs_storage};

use super::*;

pub(crate) async fn delete_card(user: &User<'_>, card: &Card<'_>) -> sqlx::Result<bool> {
    if !card.is_editable(user) {
        return Err(sqlx::Error::RowNotFound);
    }

    let board = card.board().await?;

    let db_pool = db_pool().await;

    sqlx::query!(
        "DELETE FROM cards WHERE id = $1",
        card.id, // $1
    )
    .execute(db_pool)
    .await?;

    jobs_storage()
        .await
        .push_activity(user, &board, ActivityAction::DeleteCard, card, &())
        .await;

    Ok(true)
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ list.id }"#,
    ty = "AsyncRedisCache<Uuid, Vec<Card<'_>>>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_ALL_CARDS).await }"##
)]
pub async fn get_all_cards<'a>(list: &List<'_>) -> sqlx::Result<Vec<Card<'a>>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Card,
        "SELECT * FROM cards WHERE list_id = $1 ORDER BY position ASC",
        list.id, // $1
    )
    .fetch_all(db_pool)
    .await
}

pub async fn get_card_by_id<'a>(id: Uuid) -> sqlx::Result<Card<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Card,
        r#"SELECT * FROM cards WHERE id = $1 LIMIT 1"#,
        id, // $1
    )
    .fetch_one(db_pool)
    .await
}

pub async fn get_visible_card_by_id<'a>(id: Uuid, user: Option<&User<'_>>) -> sqlx::Result<Card<'a>> {
    let card = get_card_by_id(id).await?;

    if card.is_visible(user).await? {
        Ok(card)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

pub async fn insert_card<'a>(user: &User<'_>, params: CardParams) -> ValidationResult<Card<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    let list = get_visible_list_by_id(params.list_id, Some(user))
        .await
        .or_validation_errors_with("list_id", ERROR_IS_INVALID.clone())?;

    if !list.can_create_card(user).await.or_validation_errors()? {
        validation_errors.add("list_id", ERROR_IS_INVALID.clone());

        return Err(validation_errors);
    }

    let labels = get_visible_labels_by_ids(&params.label_ids, Some(user))
        .await
        .or_validation_errors_with("label_ids", ERROR_IS_INVALID.clone())?;

    let board = list.board().await.or_validation_errors()?;

    let content = params.content.trim();
    let position = suggest_card_position(&list).await;

    let db_pool = db_pool().await;

    let card = sqlx::query_as!(
        Card,
        "INSERT INTO cards (list_id, user_id, content, position) VALUES ($1, $2, $3, $4) RETURNING *",
        list.id,  // $1
        user.id,  // $2
        content,  // $3
        position, // $4
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    let _ = insert_card_labels(&card, &labels).await;

    jobs_storage()
        .await
        .push_activity(user, &board, ActivityAction::CreateCard, &card, &card)
        .await;

    Ok(card)
}

async fn remove_all_cards_cache(list: &List<'_>) {
    GET_ALL_CARDS.cache_remove(CACHE_PREFIX_GET_ALL_CARDS, &list.id).await;
}

async fn suggest_card_position(list: &List<'_>) -> i16 {
    let db_pool = db_pool().await;

    sqlx::query_scalar!(
        r#"SELECT MAX(position) as "max_position!" FROM cards WHERE list_id = $1 LIMIT 1"#,
        list.id
    )
    .fetch_one(db_pool)
    .await
    .map(|max| max + 1)
    .unwrap_or(0)
}

pub async fn paginate_cards<'a>(cursor_params: CursorParams, list: &List<'_>) -> CursorPage<Card<'a>> {
    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &Card| node.id,
        async |after| get_card_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let cursor_position = cursor_resource.map(|c| c.position);

            sqlx::query_as!(
                Card,
                "SELECT * FROM cards WHERE ($1::smallint IS NULL OR position > $1) AND list_id = $2
                ORDER BY position ASC
                LIMIT $3",
                cursor_position, // $1
                list.id,         // $2
                limit,           // $3
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}

pub async fn update_card<'a>(user: &User<'_>, card: &Card<'a>, params: CardParams) -> ValidationResult<Card<'a>> {
    params.validate()?;

    let mut params_label_ids = params.label_ids.clone();
    let mut card_label_ids = card.all_label_ids().await.unwrap_or_default();

    params_label_ids.sort();
    card_label_ids.sort();

    if params.list_id == card.list_id && params.content == card.content && params_label_ids == card_label_ids {
        return Ok(card.clone());
    }

    let mut validation_errors = ValidationErrors::new();

    if !card.is_editable(user) {
        return Err(validation_errors);
    }

    let mut position = card.position;

    let list = card.list().await.or_validation_errors()?;

    let new_list = if card.list_id != params.list_id {
        let new_list = get_visible_list_by_id(params.list_id, Some(user))
            .await
            .or_validation_errors_with("list_id", ERROR_IS_INVALID.clone())?;

        if !card.is_movable(user).await.or_validation_errors()?
            || !new_list.can_move_card(user).await.or_validation_errors()?
            || list.board_id != new_list.board_id
        {
            validation_errors.add("list_id", ERROR_IS_INVALID.clone());

            return Err(validation_errors);
        }

        position = suggest_card_position(&new_list).await;

        Some(new_list)
    } else {
        None
    };

    let labels = get_visible_labels_by_ids(&params.label_ids, Some(user))
        .await
        .or_validation_errors_with("label_ids", ERROR_IS_INVALID.clone())?;

    let board = card.board().await.or_validation_errors()?;

    let content = params.content.trim();

    let db_pool = db_pool().await;

    let updated_card = sqlx::query_as!(
        Card,
        "UPDATE cards SET list_id = $2, content = $3, position = $4 WHERE id = $1 RETURNING *",
        card.id,        // $1
        params.list_id, // $2
        content,        // $3
        position,       // $4
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    let _ = update_card_labels(&updated_card, &labels).await;

    remove_all_cards_cache(&list).await;

    if let Some(new_list) = new_list {
        remove_all_cards_cache(&new_list).await;
    }

    jobs_storage()
        .await
        .push_activity(user, &board, ActivityAction::UpdateCard, &updated_card, &updated_card)
        .await;

    Ok(updated_card)
}

pub async fn update_card_list<'a>(
    user: &User<'_>,
    card: &Card<'_>,
    new_list: &List<'_>,
    position: i16,
) -> ValidationResult<Card<'a>> {
    let list = card.list().await.or_validation_errors()?;

    if !card.is_movable(user).await.or_validation_errors()?
        || !new_list.can_move_card(user).await.or_validation_errors()?
        || list.board_id != new_list.board_id
        || list.id == new_list.id
        || position < 0
    {
        return Err(ValidationErrors::new());
    }

    let board = card.board().await.or_validation_errors()?;

    let mut transaction = db_pool().await.begin().await.or_validation_errors()?;

    sqlx::query!("SET CONSTRAINTS ALL DEFERRED")
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;

    sqlx::query!(
        "UPDATE cards SET position = position + 1 WHERE list_id = $1 AND position >= $2",
        new_list.id, // $1
        position,    // $2
    )
    .execute(&mut *transaction)
    .await
    .or_validation_errors()?;

    let updated_card = sqlx::query_as!(
        Card,
        "UPDATE cards SET list_id = $1, position = $2 WHERE id = $3 RETURNING *",
        new_list.id, // $1
        position,    // $2
        card.id,     // $3
    )
    .fetch_one(&mut *transaction)
    .await
    .or_validation_errors()?;

    sqlx::query!(
        "UPDATE cards SET position = position - 1 WHERE list_id = $1 AND position > $2",
        list.id,       // $1
        card.position, // $2
    )
    .execute(&mut *transaction)
    .await
    .or_validation_errors()?;

    transaction.commit().await.or_validation_errors()?;

    remove_all_cards_cache(&list).await;
    remove_all_cards_cache(new_list).await;

    jobs_storage()
        .await
        .push_activity(
            user,
            &board,
            ActivityAction::UpdateCardList,
            &updated_card,
            &updated_card,
        )
        .await;

    Ok(updated_card)
}

pub async fn update_card_position<'a>(user: &User<'_>, card: &Card<'_>, position: i16) -> ValidationResult<Card<'a>> {
    if !card.is_movable(user).await.or_validation_errors()? || position < 0 || position == card.position {
        return Err(ValidationErrors::new());
    }

    let board = card.board().await.or_validation_errors()?;
    let list = card.list().await.or_validation_errors()?;

    let mut transaction = db_pool().await.begin().await.or_validation_errors()?;

    sqlx::query!("SET CONSTRAINTS ALL DEFERRED")
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;

    sqlx::query!("UPDATE cards SET position = -1 WHERE id = $1", card.id)
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;

    if position > card.position {
        sqlx::query!(
            "UPDATE cards SET position = position - 1 WHERE list_id = $1 AND position BETWEEN $2 AND $3",
            card.list_id,      // $1
            card.position + 1, // $2
            position,          // $3
        )
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;
    } else {
        sqlx::query!(
            "UPDATE cards SET position = position + 1 WHERE list_id = $1 AND position BETWEEN $2 AND $3",
            card.list_id,      // $1
            position,          // $2
            card.position - 1, // $3
        )
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;
    }

    let updated_card = sqlx::query_as!(
        Card,
        "UPDATE cards SET position = $1 WHERE id = $2 RETURNING *",
        position, // $1
        card.id,  // $2
    )
    .fetch_one(&mut *transaction)
    .await
    .or_validation_errors()?;

    transaction.commit().await.or_validation_errors()?;

    remove_all_cards_cache(&list).await;

    jobs_storage()
        .await
        .push_activity(
            user,
            &board,
            ActivityAction::UpdateCardPosition,
            &updated_card,
            &updated_card,
        )
        .await;

    Ok(updated_card)
}
