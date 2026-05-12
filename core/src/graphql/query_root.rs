use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

use crate::graphql::objects::{BoardObject, InfoObject, ListObject, UserObject};
use crate::graphql::{CustomContext, IDExt};
use crate::pagination::CursorParams;
use crate::{Info, commands};

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

    async fn current_user<'a>(&self, ctx: &'a Context<'_>) -> Option<UserObject<'a>> {
        ctx.user_opt().map(|user| UserObject(user.clone()))
    }

    async fn info(&self) -> InfoObject {
        InfoObject(Info::default())
    }

    async fn list(&self, ctx: &Context<'_>, id: ID) -> Result<Option<ListObject<'_>>> {
        let id = id.try_into_uuid()?;
        let user = ctx.user_opt();

        let list = commands::get_list_by_id(id).await?;

        if list.is_visible(user).await? {
            Ok(Some(ListObject(list)))
        } else {
            Ok(None)
        }
    }

    async fn user(&self, username: String) -> Option<UserObject<'_>> {
        commands::get_user_by_username(&username).await.map(UserObject).ok()
    }
}
