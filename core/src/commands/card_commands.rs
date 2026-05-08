use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use toolbox::constants::ERROR_IS_INVALID;
use toolbox::pagination::{CursorPage, CursorParams};
use toolbox::validator::{OrValidationErrors, ValidationResult};

use crate::db_pool;
use crate::models::{Card, List, User};
use crate::params::CardParams;

use super::get_list_by_id;

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

pub async fn insert_card<'a>(user: &User, params: CardParams) -> ValidationResult<Card<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    let list = get_list_by_id(params.list_id).await.or_validation_errors()?;

    if !list.is_editable(Some(user)) {
        validation_errors.add("list_id", ERROR_IS_INVALID.clone());

        return Err(validation_errors);
    }

    let position = suggest_card_position(&list).await;

    let db_pool = db_pool().await;

    sqlx::query_as!(
        Card,
        "INSERT INTO cards (list_id, user_id, content, position) VALUES ($1, $2, $3, $4) RETURNING *",
        list.id,        // $1
        user.id,        // $2
        params.content, // $3
        position,       // $4
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()
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

pub async fn paginate_cards<'a>(cursor_params: CursorParams, list: &List<'a>) -> CursorPage<Card<'a>> {
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

pub async fn update_card_list<'a>(
    user: &User,
    card: &Card<'_>,
    new_list: &List<'_>,
    position: i16,
) -> ValidationResult<Card<'a>> {
    let list = card.list().await.or_validation_errors()?;

    if !list.is_editable(Some(user))
        || !new_list.is_editable(Some(user))
        || list.board_id != new_list.board_id
        || list.id == new_list.id
        || position < 0
    {
        return Err(ValidationErrors::new());
    }

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

    Ok(updated_card)
}

pub async fn update_card_position<'a>(user: &User, card: &Card<'_>, position: i16) -> ValidationResult<Card<'a>> {
    let list = card.list().await.or_validation_errors()?;

    if !list.is_editable(Some(user)) || position < 0 || position == card.position {
        return Err(ValidationErrors::new());
    }

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
            list.id,           // $1
            card.position + 1, // $2
            position,          // $3
        )
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;
    } else {
        sqlx::query!(
            "UPDATE cards SET position = position + 1 WHERE list_id = $1 AND position BETWEEN $2 AND $3",
            list.id,           // $1
            position,          // $2
            card.position - 1, // $3
        )
        .execute(&mut *transaction)
        .await
        .or_validation_errors()?;
    }

    let card = sqlx::query_as!(
        Card,
        "UPDATE cards SET position = $1 WHERE id = $2 RETURNING *",
        position, // $1
        card.id,  // $2
    )
    .fetch_one(&mut *transaction)
    .await
    .or_validation_errors()?;

    transaction.commit().await.or_validation_errors()?;

    Ok(card)
}
