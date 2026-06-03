use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, NaiveDate, Utc};
use url::Url;
use uuid::Uuid;

use crate::commands;
use crate::enums::CountryCode;
use crate::graphql::CustomContext;
use crate::models::User;
use crate::pagination::CursorParams;

use super::BoardObject;

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

    async fn full_name(&self) -> &str {
        &self.0.full_name
    }

    async fn birthdate(&self) -> NaiveDate {
        self.0.birthdate
    }

    async fn country_code(&self) -> CountryCode {
        self.0.country_code
    }

    async fn avatar_image_url(&self) -> Url {
        self.0.avatar_image_url()
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
