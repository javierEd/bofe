use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::enums::ActivityAction;

#[cfg(feature = "graphql")]
use crate::commands;

#[cfg(feature = "graphql")]
use super::{Board, Card, List, User};

#[derive(Clone, Deserialize, Serialize)]
pub struct Activity {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub board_id: Uuid,
    pub action: ActivityAction,
    pub target_id: Uuid,
    pub data: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "graphql")]
impl Activity {
    pub async fn user(&self) -> Option<User<'_>> {
        if let Some(user_id) = self.user_id {
            commands::get_user_by_id(user_id).await.ok()
        } else {
            None
        }
    }

    pub async fn board(&self) -> sqlx::Result<Board<'_>> {
        commands::get_board_by_id(self.board_id).await
    }
}

#[cfg(feature = "graphql")]
pub trait ActivityExt<T> {
    fn data(&self) -> Option<T>;
}

#[cfg(feature = "graphql")]
impl<'a> ActivityExt<Board<'a>> for Activity {
    fn data(&self) -> Option<Board<'a>> {
        match self.action {
            ActivityAction::CreateBoard | ActivityAction::UpdateBoard => self
                .data
                .clone()
                .and_then(|data| serde_json::from_value::<Board>(data).ok()),
            _ => None,
        }
    }
}

#[cfg(feature = "graphql")]
impl<'a> ActivityExt<Card<'a>> for Activity {
    fn data(&self) -> Option<Card<'a>> {
        match self.action {
            ActivityAction::CreateCard
            | ActivityAction::UpdateCard
            | ActivityAction::UpdateCardList
            | ActivityAction::UpdateCardPosition => self
                .data
                .clone()
                .and_then(|data| serde_json::from_value::<Card>(data).ok()),
            _ => None,
        }
    }
}

#[cfg(feature = "graphql")]
impl<'a> ActivityExt<List<'a>> for Activity {
    fn data(&self) -> Option<List<'a>> {
        match self.action {
            ActivityAction::CreateList | ActivityAction::UpdateList | ActivityAction::UpdateListPosition => self
                .data
                .clone()
                .and_then(|data| serde_json::from_value::<List>(data).ok()),
            _ => None,
        }
    }
}
