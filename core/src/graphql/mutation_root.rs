use async_graphql::dynamic::indexmap::IndexMap;
use async_graphql::{Context, Error, ErrorExtensions, Name, Object, Result, Value};
use uuid::Uuid;
use validator::ValidationErrors;

use crate::commands;
use crate::graphql::CustomContext;
use crate::graphql::guards::{GuestGuard, UserGuard};
use crate::graphql::objects::{BoardObject, CardObject, ListObject, SessionObject, UserObject};
use crate::params::{BoardParams, CardParams, ListParams, SessionParams, UpdateListParams, UserParams};

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

    #[graphql(guard = "GuestGuard")]
    async fn create_session(&self, ctx: &Context<'_>, params: SessionParams) -> Result<SessionObject<'_>> {
        let application = ctx.application();
        let ip_address = ctx.client_ip();

        commands::insert_session(application, ip_address, params)
            .await
            .map(SessionObject)
            .map_err(|errors| to_mutation_error("Failed to create session", errors))
    }

    #[graphql(guard = "GuestGuard")]
    async fn create_user(&self, params: UserParams) -> Result<UserObject<'_>> {
        commands::insert_user(params)
            .await
            .map(UserObject)
            .map_err(|errors| to_mutation_error("Failed to create user", errors))
    }

    #[graphql(guard = "UserGuard")]
    async fn delete_board(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let user = ctx.user();
        let board = commands::get_board_by_id(id).await?;

        commands::delete_board(user, &board)
            .await
            .map_err(|_| to_mutation_error("Failed to delete board", ValidationErrors::new()))
    }

    #[graphql(guard = "UserGuard")]
    async fn finish_session(&self, ctx: &Context<'_>) -> Result<bool> {
        let session = ctx.session();

        commands::finish_session(session)
            .await
            .map_err(|_| to_mutation_error("Failed to finish session", ValidationErrors::new()))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_board(&self, ctx: &Context<'_>, id: Uuid, params: BoardParams) -> Result<BoardObject<'_>> {
        let user = ctx.user();
        let board = commands::get_board_by_id(id).await?;

        commands::update_board(user, &board, params)
            .await
            .map(BoardObject)
            .map_err(|errors| to_mutation_error("Failed to update board", errors))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_card_list(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        list_id: Uuid,
        position: i16,
    ) -> Result<CardObject<'_>> {
        let user = ctx.user();
        let card = commands::get_card_by_id(id).await?;
        let new_list = commands::get_list_by_id(list_id).await?;

        commands::update_card_list(user, &card, &new_list, position)
            .await
            .map(CardObject)
            .map_err(|errors| to_mutation_error("Failed to update card list", errors))
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
    async fn update_list(&self, ctx: &Context<'_>, id: Uuid, params: UpdateListParams) -> Result<ListObject<'_>> {
        let user = ctx.user();
        let list = commands::get_list_by_id(id).await?;

        commands::update_list(user, &list, params)
            .await
            .map(ListObject)
            .map_err(|errors| to_mutation_error("Failed to update list", errors))
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
