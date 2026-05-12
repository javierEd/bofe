use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::enums::BoardVisibility;
use crate::graphql::CustomContext;
use crate::models::{Board, Card, List, Session, User};
use crate::pagination::CursorParams;
use crate::{Info, commands};

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

    async fn is_editable(&self, ctx: &Context<'_>) -> bool {
        self.0.is_editable(ctx.user_opt())
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct CardObject<'a>(pub Card<'a>);

#[Object]
impl CardObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn list_id(&self) -> Uuid {
        self.0.list_id
    }

    async fn content(&self) -> &str {
        &self.0.content
    }

    async fn position(&self) -> i16 {
        self.0.position
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct InfoObject(pub Info);

#[Object]
impl InfoObject {
    async fn built_at(&self) -> DateTime<Utc> {
        self.0.built_at
    }

    async fn version(&self) -> &str {
        &self.0.version
    }
}

pub struct ListObject<'a>(pub List<'a>);

#[Object]
impl ListObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn position(&self) -> i16 {
        self.0.position
    }

    async fn cards(
        &self,
        after: Option<Uuid>,
        first: Option<i32>,
    ) -> Result<Connection<Uuid, CardObject<'_>, EmptyFields, EmptyFields>> {
        query(
            after.map(|a| a.to_string()),
            None,
            first,
            None,
            |after, _before, first, _last| async move {
                let first = first.map(|v| v as u8).unwrap_or(10);
                let cursor_page = commands::paginate_cards(CursorParams::new(after, first), &self.0).await;
                let mut connection = Connection::new(false, cursor_page.has_next_page);

                connection.edges.extend(
                    cursor_page
                        .nodes
                        .into_iter()
                        .map(|card| Edge::new(card.id, CardObject(card))),
                );

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct SessionObject<'a>(pub Session<'a>);

#[Object]
impl SessionObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn token(&self) -> &str {
        &self.0.token
    }

    async fn expires_at(&self) -> DateTime<Utc> {
        self.0.expires_at
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct UserObject<'a>(pub User<'a>);

#[Object]
impl UserObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn username(&self) -> &str {
        &self.0.username
    }

    async fn initials(&self) -> String {
        self.0.initials()
    }

    async fn display_name(&self) -> &str {
        &self.0.display_name
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
                let cursor_page =
                    commands::paginate_boards(CursorParams::new(after, first), Some(&self.0), target_user).await;
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

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
