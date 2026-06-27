use async_graphql::connection::{Connection, Edge, EmptyFields, query};
use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::graphql::context::CustomExt;
use crate::models::{Attachment, Confirmation, List, Member, Session};
use crate::pagination::CursorParams;
use crate::{Info, commands};

mod activity_object;
mod board_object;
mod card_object;
mod label_object;
mod user_object;

pub use activity_object::ActivityObject;
pub use board_object::BoardObject;
pub use card_object::CardObject;
pub use label_object::LabelObject;
pub use user_object::UserObject;

pub struct AttachmentObject<'a>(pub Attachment<'a>);

#[Object]
impl AttachmentObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn thumbnail_url(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 256)] width: u16,
        #[graphql(default = 256)] height: u16,
    ) -> Result<Option<Url>> {
        let user = ctx.user_opt();

        Ok(self.0.thumbnail_url(user, width, height).await?)
    }

    async fn url(&self, ctx: &Context<'_>) -> Result<Url> {
        let user = ctx.user_opt();

        Ok(self.0.url(user).await?)
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }
}

pub struct ConfirmationObject<'a>(pub Confirmation<'a>);

#[Object]
impl ConfirmationObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
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

    async fn all_cards(&self) -> Result<Vec<CardObject<'_>>> {
        Ok(commands::get_all_cards(&self.0)
            .await?
            .into_iter()
            .map(CardObject)
            .collect())
    }

    async fn can_create_card(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(user) = ctx.user_opt()
            && self.0.can_create_card(user).await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn can_move_card(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(user) = ctx.user_opt()
            && self.0.can_move_card(user).await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn is_editable(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(user) = ctx.user_opt()
            && self.0.is_editable(user).await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn is_movable(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(user) = ctx.user_opt()
            && self.0.is_movable(user).await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}

pub struct MemberObject(pub Member);

#[Object]
impl MemberObject {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn user(&self) -> Result<UserObject<'_>> {
        Ok(self.0.user().await.map(UserObject)?)
    }

    async fn is_admin(&self) -> bool {
        self.0.is_admin
    }

    async fn is_editable(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(user) = ctx.user_opt()
            && self.0.is_editable(user).await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn is_removable(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(user) = ctx.user_opt()
            && self.0.is_removable(user).await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
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

    async fn user(&self) -> Result<UserObject<'_>> {
        Ok(self.0.user().await.map(UserObject)?)
    }

    async fn token(&self) -> &str {
        &self.0.token
    }

    async fn expires_at(&self) -> DateTime<Utc> {
        self.0.expires_at
    }

    async fn refreshed_at(&self) -> Option<DateTime<Utc>> {
        self.0.refreshed_at
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
