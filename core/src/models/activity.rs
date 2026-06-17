use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::commands;
use crate::enums::ActivityAction;

use super::{Board, Card, List, User};

#[derive(Clone, Deserialize, Serialize)]
pub struct Activity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub board_id: Uuid,
    pub action: ActivityAction,
    pub target_id: Uuid,
    pub data: Option<Value>,
    pub created_at: DateTime<Utc>,
}

impl Activity {
    pub async fn user(&self) -> sqlx::Result<User<'_>> {
        commands::get_user_by_id(self.user_id).await
    }

    pub async fn board(&self) -> sqlx::Result<Board<'_>> {
        commands::get_board_by_id(self.board_id).await
    }
}

pub trait ActivityExt<T> {
    fn data(&self) -> Option<T>;
}

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
