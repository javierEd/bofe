use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};

use toolbox::graphql::objects::IdentityUserObject;
use toolbox::pagination::CursorParams;
use uuid::Uuid;

use crate::Info;
use crate::commands;
use crate::graphql::CustomContext;
use crate::models::{Board, User};

pub struct BoardObject<'a>(pub Board<'a>);

#[Object]
impl BoardObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn user(&self) -> Result<UserObject> {
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

pub struct UserObject(pub User);

#[Object]
impl UserObject {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn identity_user(&self, ctx: &Context<'_>) -> Result<IdentityUserObject<'_>> {
        Ok(self
            .0
            .identity_user(ctx.identity_client())
            .await
            .map(IdentityUserObject)?)
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
                    commands::paginate_boards(CursorParams { after, first }, Some(&self.0), target_user).await;
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
