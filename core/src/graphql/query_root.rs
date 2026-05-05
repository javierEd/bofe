use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, Object, Result};
use toolbox::pagination::CursorParams;
use uuid::Uuid;

use crate::Info;
use crate::commands;
use crate::graphql::CustomContext;
use crate::graphql::objects::{BoardObject, InfoObject, UserObject};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn board(&self, ctx: &Context<'_>, id_or_slug: String) -> Option<BoardObject<'_>> {
        let user = ctx.user_opt();

        commands::get_board_by_id_or_slug(&id_or_slug, user)
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
                let cursor_page = commands::paginate_boards(CursorParams { after, first }, None, target_user).await;
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

    async fn current_user(&self, ctx: &Context<'_>) -> Option<UserObject> {
        ctx.user_opt().map(|user| UserObject(user.clone()))
    }

    async fn info(&self) -> InfoObject {
        InfoObject(Info::default())
    }

    async fn user(&self, ctx: &Context<'_>, username: String) -> Option<UserObject> {
        let identity_user = commands::get_identity_user(ctx.identity_client(), &username)
            .await
            .ok()?;

        commands::get_user_by_identity_user(&identity_user)
            .await
            .map(UserObject)
            .ok()
    }
}
