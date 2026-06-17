use async_graphql::{ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::enums::ActivityAction;
use crate::models::{Activity, ActivityExt};

use super::{BoardObject, CardObject, ListObject, UserObject};

pub struct ActivityObject(pub Activity);

#[Object]
impl ActivityObject {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn user(&self) -> Result<UserObject<'_>> {
        Ok(self.0.user().await.map(UserObject)?)
    }

    async fn board(&self) -> Result<BoardObject<'_>> {
        Ok(self.0.board().await.map(BoardObject)?)
    }

    async fn action(&self) -> ActivityAction {
        self.0.action
    }

    async fn board_data(&self) -> Option<BoardObject<'_>> {
        self.0.data().map(BoardObject)
    }

    async fn card_data(&self) -> Option<CardObject<'_>> {
        self.0.data().map(CardObject)
    }

    async fn list_data(&self) -> Option<ListObject<'_>> {
        self.0.data().map(ListObject)
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }
}
