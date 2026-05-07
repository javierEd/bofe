use async_graphql::dynamic::indexmap::IndexMap;
use async_graphql::{Context, Error, ErrorExtensions, Name, Object, Result, Value};
use uuid::Uuid;
use validator::ValidationErrors;

use crate::commands;
use crate::graphql::CustomContext;
use crate::graphql::guards::UserGuard;
use crate::graphql::objects::{BoardObject, CardObject, ListObject};
use crate::params::{BoardParams, CardParams, ListParams};

pub struct MutationRoot;

fn to_mutation_error(message: &str, errors: ValidationErrors) -> Error {
    let params_err = Value::Object(
        errors
            .field_errors()
            .iter()
            .map(|(field, error)| {
                let mut details = IndexMap::new();

                details.insert(Name::new("code"), Value::from(error[0].code.clone()));
                details.insert(Name::new("message"), Value::from(error[0].message.clone()));

                (Name::new(field), Value::from(details))
            })
            .collect(),
    );

    Error::new(message).extend_with(|_, e| e.set("params", params_err))
}

#[Object]
impl MutationRoot {
    #[graphql(guard = "UserGuard")]
    async fn create_board(&self, ctx: &Context<'_>, params: BoardParams) -> Result<BoardObject<'_>> {
        let user = ctx.user();

        commands::insert_board(user, params)
            .await
            .map(BoardObject)
            .map_err(|errors| to_mutation_error("Failed to create board", errors))
    }

    #[graphql(guard = "UserGuard")]
    async fn create_card(&self, ctx: &Context<'_>, params: CardParams) -> Result<CardObject<'_>> {
        let user = ctx.user();

        commands::insert_card(user, params)
            .await
            .map(CardObject)
            .map_err(|errors| to_mutation_error("Failed to create card", errors))
    }

    #[graphql(guard = "UserGuard")]
    async fn create_list(&self, ctx: &Context<'_>, params: ListParams) -> Result<ListObject<'_>> {
        let user = ctx.user();

        commands::insert_list(user, params)
            .await
            .map(ListObject)
            .map_err(|errors| to_mutation_error("Failed to create list", errors))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_card_position(&self, ctx: &Context<'_>, id: Uuid, position: i16) -> Result<CardObject<'_>> {
        let user = ctx.user();
        let card = commands::get_card_by_id(id).await?;

        commands::update_card_position(user, &card, position)
            .await
            .map(CardObject)
            .map_err(|errors| to_mutation_error("Failed to update card position", errors))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_list_position(&self, ctx: &Context<'_>, id: Uuid, position: i16) -> Result<ListObject<'_>> {
        let user = ctx.user();
        let list = commands::get_list_by_id(id).await?;

        commands::update_list_position(user, &list, position)
            .await
            .map(ListObject)
            .map_err(|errors| to_mutation_error("Failed to update list position", errors))
    }
}
