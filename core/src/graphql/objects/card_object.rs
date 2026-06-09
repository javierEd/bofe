use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::graphql::context::CustomExt;
use crate::models::Card;

use super::{BoardObject, LabelObject, ListObject, UserObject};

pub struct CardObject<'a>(pub Card<'a>);

#[Object]
impl CardObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn board(&self) -> Result<BoardObject<'_>> {
        Ok(self.0.board().await.map(BoardObject)?)
    }

    async fn list(&self) -> Result<ListObject<'_>> {
        Ok(self.0.list().await.map(ListObject)?)
    }

    async fn content(&self, max_length: Option<u16>, strip_markdown: Option<bool>) -> String {
        self.0.content(max_length, strip_markdown)
    }

    async fn all_labels(&self) -> Result<Vec<LabelObject<'_>>> {
        Ok(self.0.all_labels().await?.into_iter().map(LabelObject).collect())
    }

    async fn position(&self) -> i16 {
        self.0.position
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

    async fn is_movable(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(user) = ctx.user_opt()
            && self.0.is_movable(user).await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn user(&self) -> Result<UserObject<'_>> {
        Ok(self.0.user().await.map(UserObject)?)
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
