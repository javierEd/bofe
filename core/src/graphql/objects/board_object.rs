use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::commands;
use crate::enums::BoardVisibility;
use crate::graphql::context::CustomExt;
use crate::models::Board;
use crate::pagination::CursorParams;

use super::{LabelObject, ListObject, MemberObject, UserObject};

pub struct BoardObject<'a>(pub Board<'a>);

#[Object]
impl BoardObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn user(&self) -> Result<UserObject<'_>> {
        Ok(self.0.user().await.map(UserObject)?)
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn slug(&self) -> &str {
        &self.0.slug
    }

    async fn description(&self) -> &str {
        &self.0.description
    }

    async fn visibility(&self) -> BoardVisibility {
        self.0.visibility
    }

    async fn labels(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
    ) -> Result<Connection<Uuid, LabelObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_labels(CursorParams::new(after, first), &self.0).await;
                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|label| Edge::new(label.id, LabelObject(label))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn all_labels(&self) -> Result<Vec<LabelObject<'_>>> {
        Ok(commands::get_all_labels(&self.0)
            .await?
            .into_iter()
            .map(LabelObject)
            .collect())
    }

    async fn lists(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
    ) -> Result<Connection<Uuid, ListObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_lists(CursorParams::new(after, first), &self.0).await;
                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|list| Edge::new(list.id, ListObject(list))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn all_lists(&self) -> Result<Vec<ListObject<'_>>> {
        Ok(commands::get_all_lists(&self.0)
            .await?
            .into_iter()
            .map(ListObject)
            .collect())
    }

    async fn members(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
    ) -> Result<Connection<Uuid, MemberObject, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_members(CursorParams::new(after, first), &self.0).await;
                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|member| Edge::new(member.id, MemberObject(member))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn can_create_card(&self, ctx: &Context<'_>) -> bool {
        if let Some(user) = ctx.user_opt()
            && self.0.can_create_card(user).await
        {
            true
        } else {
            false
        }
    }

    async fn can_create_label(&self, ctx: &Context<'_>) -> bool {
        if let Some(user) = ctx.user_opt()
            && self.0.can_create_label(user)
        {
            true
        } else {
            false
        }
    }

    async fn can_create_list(&self, ctx: &Context<'_>) -> bool {
        if let Some(user) = ctx.user_opt()
            && self.0.can_create_list(user)
        {
            true
        } else {
            false
        }
    }

    async fn can_create_member(&self, ctx: &Context<'_>) -> bool {
        if let Some(user) = ctx.user_opt()
            && self.0.can_create_member(user)
        {
            true
        } else {
            false
        }
    }

    async fn can_move_card(&self, ctx: &Context<'_>) -> bool {
        if let Some(user) = ctx.user_opt()
            && self.0.can_move_card(user).await
        {
            true
        } else {
            false
        }
    }

    async fn can_move_list(&self, ctx: &Context<'_>) -> bool {
        if let Some(user) = ctx.user_opt()
            && self.0.can_move_list(user)
        {
            true
        } else {
            false
        }
    }

    async fn is_editable(&self, ctx: &Context<'_>) -> bool {
        if let Some(user) = ctx.user_opt()
            && self.0.is_editable(user)
        {
            true
        } else {
            false
        }
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
