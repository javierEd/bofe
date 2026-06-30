use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::commands;
use crate::graphql::IDExt;
use crate::graphql::context::CustomExt;
use crate::models::Card;

use super::{AttachmentObject, BoardObject, LabelObject, ListObject, UserObject};

pub struct CardObject<'a>(pub Card<'a>);

#[Object]
impl CardObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn user(&self) -> Option<UserObject<'_>> {
        self.0.user().await.map(UserObject)
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

    async fn attachment(&self, id: ID) -> Result<Option<AttachmentObject<'_>>> {
        let id = id.try_into_uuid()?;

        let Ok(card_attachment) = commands::get_card_attachment(self.0.id, id).await else {
            return Ok(None);
        };

        Ok(Some(card_attachment.attachment().await.map(AttachmentObject)?))
    }

    async fn all_attachments(&self) -> Result<Vec<AttachmentObject<'_>>> {
        Ok(self
            .0
            .all_attachments()
            .await?
            .into_iter()
            .map(AttachmentObject)
            .collect())
    }

    async fn all_labels(&self) -> Result<Vec<LabelObject<'_>>> {
        Ok(self.0.all_labels().await?.into_iter().map(LabelObject).collect())
    }

    async fn attachments_count(&self) -> Result<i64> {
        Ok(self.0.attachments_count().await?)
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

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
