use async_graphql::dynamic::indexmap::IndexMap;
use async_graphql::{Context, Error, ErrorExtensions, InputType, Name, Object, Result, Value};
use convert_case::{Case, Casing};
use uuid::Uuid;
use validator::ValidationErrors;

use crate::commands;
use crate::constants::*;
use crate::enums::ConfirmationAction;
use crate::graphql::context::CustomExt;
use crate::graphql::guards::{GuestGuard, UserGuard};
use crate::graphql::objects::*;
use crate::params::*;

pub struct MutationRoot;

fn to_mutation_error(message: &str, errors: Option<ValidationErrors>) -> Error {
    let error = Error::new(message);

    if let Some(errors) = errors {
        let params_err = Value::Object(
            errors
                .field_errors()
                .iter()
                .map(|(field, errors)| {
                    let error = &errors[0];
                    let mut details = IndexMap::new();
                    let mut details_args = IndexMap::new();

                    for (key, value) in error.params.iter() {
                        details_args.insert(Name::new(key.to_case(Case::Camel)), value.to_value());
                    }

                    details.insert(Name::new("code"), Value::from(error.code.clone()));
                    details.insert(Name::new("message"), Value::from(error.message.clone()));
                    details.insert(Name::new("args"), Value::Object(details_args));

                    (Name::new(field.to_case(Case::Camel)), Value::from(details))
                })
                .collect(),
        );

        error.extend_with(|_, e| e.set("params", params_err))
    } else {
        error
    }
}

#[Object]
impl MutationRoot {
    #[graphql(guard = "UserGuard")]
    async fn confirm_email(
        &self,
        ctx: &Context<'_>,
        confirmation_params: ConfirmationParams,
    ) -> Result<UserObject<'_>> {
        let user = ctx.user();
        let l10n = ctx.l10n();

        commands::confirm_user_email(user, confirmation_params)
            .await
            .map(UserObject)
            .map_err(|_| to_mutation_error(&l10n.text(KEY_TEXT_FAILED_TO_CONFIRM_EMAIL), None))
    }

    #[graphql(guard = "UserGuard")]
    async fn create_board(&self, ctx: &Context<'_>, params: BoardParams) -> Result<BoardObject<'_>> {
        let user = ctx.user();

        commands::insert_board(user, params)
            .await
            .map(BoardObject)
            .map_err(|errors| to_mutation_error("Failed to create board", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn create_card(&self, ctx: &Context<'_>, params: CardParams) -> Result<CardObject<'_>> {
        let user = ctx.user();

        commands::insert_card(user, params)
            .await
            .map(CardObject)
            .map_err(|errors| to_mutation_error("Failed to create card", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn create_label(&self, ctx: &Context<'_>, params: LabelParams) -> Result<LabelObject<'_>> {
        let user = ctx.user();

        commands::insert_label(user, params)
            .await
            .map(LabelObject)
            .map_err(|errors| to_mutation_error("Failed to create label", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn create_list(&self, ctx: &Context<'_>, params: ListParams) -> Result<ListObject<'_>> {
        let user = ctx.user();

        commands::insert_list(user, params)
            .await
            .map(ListObject)
            .map_err(|errors| to_mutation_error("Failed to create list", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn create_member(&self, ctx: &Context<'_>, params: MemberParams) -> Result<MemberObject> {
        let user = ctx.user();

        commands::insert_member(user, params)
            .await
            .map(MemberObject)
            .map_err(|errors| to_mutation_error("Failed to create member", Some(errors)))
    }

    #[graphql(guard = "GuestGuard")]
    async fn create_session(&self, ctx: &Context<'_>, params: SessionParams) -> Result<SessionObject<'_>> {
        let application = ctx.application();
        let ip_address = ctx.client_ip();
        let l10n = ctx.l10n();

        commands::insert_session(application, ip_address, params)
            .await
            .map(SessionObject)
            .map_err(|errors| to_mutation_error(&l10n.text(KEY_TEXT_FAILED_TO_CREATE_SESSION), Some(errors)))
    }

    #[graphql(guard = "GuestGuard")]
    async fn create_user(&self, ctx: &Context<'_>, params: UserParams) -> Result<UserObject<'_>> {
        let l10n = ctx.l10n();

        commands::insert_user(params)
            .await
            .map(UserObject)
            .map_err(|errors| to_mutation_error(&l10n.text(KEY_TEXT_FAILED_TO_CREATE_USER), Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn delete_board(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let user = ctx.user();
        let board = commands::get_visible_board_by_id(id, Some(user)).await?;

        commands::delete_board(user, &board)
            .await
            .map_err(|_| to_mutation_error("Failed to delete board", None))
    }

    #[graphql(guard = "UserGuard")]
    async fn delete_card(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let user = ctx.user();
        let card = commands::get_card_by_id(id).await?;

        commands::delete_card(user, &card)
            .await
            .map_err(|_| to_mutation_error("Failed to delete card", None))
    }

    #[graphql(guard = "UserGuard")]
    async fn delete_label(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let user = ctx.user();
        let label = commands::get_visible_label_by_id(id, Some(user)).await?;

        commands::delete_label(user, &label)
            .await
            .map_err(|_| to_mutation_error("Failed to delete label", None))
    }

    #[graphql(guard = "UserGuard")]
    async fn delete_list(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let user = ctx.user();
        let list = commands::get_visible_list_by_id(id, Some(user)).await?;

        commands::delete_list(user, &list)
            .await
            .map_err(|_| to_mutation_error("Failed to delete list", None))
    }

    #[graphql(guard = "UserGuard")]
    async fn delete_member(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let user = ctx.user();
        let member = commands::get_member_by_id(id).await?;

        commands::delete_member(user, &member)
            .await
            .map_err(|_| to_mutation_error("Failed to delete member", None))
    }

    #[graphql(guard = "UserGuard")]
    async fn finish_session(&self, ctx: &Context<'_>) -> Result<bool> {
        let session = ctx.session();

        commands::finish_session(session)
            .await
            .map_err(|_| to_mutation_error("Failed to finish session", None))
    }

    #[graphql(guard = "UserGuard")]
    async fn refresh_session(&self, ctx: &Context<'_>) -> Result<SessionObject<'_>> {
        let session = ctx.session();

        commands::refresh_session(session)
            .await
            .map(SessionObject)
            .map_err(|_| to_mutation_error("Failed to refresh session", None))
    }

    #[graphql(guard = "UserGuard")]
    async fn send_email_confirmation(&self, ctx: &Context<'_>) -> Result<ConfirmationObject<'_>> {
        let user = ctx.user();
        let l10n = ctx.l10n();

        if user.email_is_confirmed() {
            return Err(to_mutation_error(&l10n.text(KEY_TEXT_EMAIL_IS_ALREADY_CONFIRMED), None));
        }

        commands::insert_confirmation(user, ConfirmationAction::Email)
            .await
            .map(ConfirmationObject)
            .map_err(|_| to_mutation_error(&l10n.text(KEY_TEXT_FAILED_TO_SEND_CONFIRMATION), None))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_board(&self, ctx: &Context<'_>, id: Uuid, params: BoardParams) -> Result<BoardObject<'_>> {
        let user = ctx.user();
        let board = commands::get_visible_board_by_id(id, Some(user)).await?;

        commands::update_board(user, &board, params)
            .await
            .map(BoardObject)
            .map_err(|errors| to_mutation_error("Failed to update board", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_card(&self, ctx: &Context<'_>, id: Uuid, params: CardParams) -> Result<CardObject<'_>> {
        let user = ctx.user();
        let card = commands::get_card_by_id(id).await?;

        commands::update_card(user, &card, params)
            .await
            .map(CardObject)
            .map_err(|errors| to_mutation_error("Failed to update card", Some(errors)))
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
        let new_list = commands::get_visible_list_by_id(list_id, Some(user)).await?;

        commands::update_card_list(user, &card, &new_list, position)
            .await
            .map(CardObject)
            .map_err(|errors| to_mutation_error("Failed to update card list", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_card_position(&self, ctx: &Context<'_>, id: Uuid, position: i16) -> Result<CardObject<'_>> {
        let user = ctx.user();
        let card = commands::get_card_by_id(id).await?;

        commands::update_card_position(user, &card, position)
            .await
            .map(CardObject)
            .map_err(|errors| to_mutation_error("Failed to update card position", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_label(&self, ctx: &Context<'_>, id: Uuid, params: UpdateLabelParams) -> Result<LabelObject<'_>> {
        let user = ctx.user();
        let label = commands::get_visible_label_by_id(id, Some(user)).await?;

        commands::update_label(user, &label, params)
            .await
            .map(LabelObject)
            .map_err(|errors| to_mutation_error("Failed to update label", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_list(&self, ctx: &Context<'_>, id: Uuid, params: UpdateListParams) -> Result<ListObject<'_>> {
        let user = ctx.user();
        let list = commands::get_visible_list_by_id(id, Some(user)).await?;

        commands::update_list(user, &list, params)
            .await
            .map(ListObject)
            .map_err(|errors| to_mutation_error("Failed to update list", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_list_position(&self, ctx: &Context<'_>, id: Uuid, position: i16) -> Result<ListObject<'_>> {
        let user = ctx.user();
        let list = commands::get_visible_list_by_id(id, Some(user)).await?;

        commands::update_list_position(user, &list, position)
            .await
            .map(ListObject)
            .map_err(|errors| to_mutation_error("Failed to update list position", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_member(&self, ctx: &Context<'_>, id: Uuid, params: UpdateMemberParams) -> Result<MemberObject> {
        let user = ctx.user();
        let member = commands::get_member_by_id(id).await?;

        commands::update_member(user, &member, params)
            .await
            .map(MemberObject)
            .map_err(|errors| to_mutation_error("Failed to update member", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_password(&self, ctx: &Context<'_>, params: UpdatePasswordParams) -> Result<UserObject<'_>> {
        let user = ctx.user();

        commands::update_user_password(user, params)
            .await
            .map(UserObject)
            .map_err(|errors| to_mutation_error("Failed to update password", Some(errors)))
    }

    #[graphql(guard = "UserGuard")]
    async fn update_profile(&self, ctx: &Context<'_>, params: UpdateProfileParams) -> Result<UserObject<'_>> {
        let user = ctx.user();

        commands::update_user_profile(user, params)
            .await
            .map(UserObject)
            .map_err(|errors| to_mutation_error("Failed to update profile", Some(errors)))
    }
}
