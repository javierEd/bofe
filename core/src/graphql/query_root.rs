use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

use crate::graphql::IDExt;
use crate::graphql::context::CustomExt;
use crate::graphql::guards::UserGuard;
use crate::graphql::objects::{BoardObject, CardObject, InfoObject, LabelObject, ListObject, UserObject};
use crate::pagination::CursorParams;
use crate::{Info, commands};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn board(&self, ctx: &Context<'_>, id: ID) -> Result<Option<BoardObject<'_>>> {
        let user = ctx.user_opt();
        let id = id.try_into_uuid()?;

        Ok(commands::get_visible_board_by_id(id, user).await.map(BoardObject).ok())
    }

    // TODO: To be removed
    #[graphql(deprecation = true)]
    async fn board_by_slug(&self, ctx: &Context<'_>, slug: String) -> Option<BoardObject<'_>> {
        let user = ctx.user_opt();

        commands::get_visible_board_by_slug(&slug, user)
            .await
            .map(BoardObject)
            .ok()
    }

    async fn boards(
        &self,
        ctx: &Context<'_>,
        after: Option<Uuid>,
        first: Option<i32>,
    ) -> Result<Connection<Uuid, BoardObject<'_>, EmptyFields, EmptyFields>> {
        let target_user = ctx.user_opt();

        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_boards(CursorParams::new(after, first), None, target_user).await;
                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|board| Edge::new(board.id, BoardObject(board))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn card(&self, ctx: &Context<'_>, id: ID) -> Result<Option<CardObject<'_>>> {
        let user = ctx.user_opt();
        let id = id.try_into_uuid()?;

        Ok(commands::get_visible_card_by_id(id, user).await.map(CardObject).ok())
    }

    async fn current_user<'a>(&self, ctx: &'a Context<'_>) -> Option<UserObject<'a>> {
        ctx.user_opt().map(|user| UserObject(user.clone()))
    }

    async fn info(&self) -> InfoObject {
        InfoObject(Info::default())
    }

    async fn label(&self, ctx: &Context<'_>, id: ID) -> Result<Option<LabelObject<'_>>> {
        let id = id.try_into_uuid()?;
        let user = ctx.user_opt();

        Ok(commands::get_visible_label_by_id(id, user).await.map(LabelObject).ok())
    }

    async fn list(&self, ctx: &Context<'_>, id: ID) -> Result<Option<ListObject<'_>>> {
        let id = id.try_into_uuid()?;
        let user = ctx.user_opt();

        Ok(commands::get_visible_list_by_id(id, user).await.map(ListObject).ok())
    }

    async fn user(&self, username: String) -> Option<UserObject<'_>> {
        commands::get_user_by_username(&username).await.map(UserObject).ok()
    }

    #[graphql(guard = "UserGuard")]
    async fn users(
        &self,
        #[graphql(name = "query")] q: String,
        after: Option<Uuid>,
        first: Option<i32>,
    ) -> Result<Connection<Uuid, UserObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_users(CursorParams::new(after, first), &q).await;
                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|user| Edge::new(user.id, UserObject(user))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}
