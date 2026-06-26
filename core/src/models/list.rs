use std::borrow::Cow;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands;
use crate::models::{Board, User};

#[derive(Clone, Deserialize, Serialize)]
pub struct List<'a> {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: Cow<'a, str>,
    pub position: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl List<'_> {
    pub async fn board<'a>(&self) -> sqlx::Result<Board<'a>> {
        commands::get_board_by_id(self.board_id).await
    }

    /// Returns true if the user can create cards on the list
    ///
    /// Only members of the board can create cards on the list
    pub async fn can_create_card(&self, user: &User<'_>) -> sqlx::Result<bool> {
        Ok(self.board().await?.can_create_card(user).await)
    }

    /// Returns true if the user can move the card
    ///
    /// Only the board owner or admin members can move cards
    pub async fn can_move_card(&self, user: &User<'_>) -> sqlx::Result<bool> {
        Ok(self.board().await?.can_move_card(user).await)
    }

    /// Returns true if the user can edit the list
    ///
    /// Only the board owner can edit the list
    pub async fn is_editable(&self, user: &User<'_>) -> sqlx::Result<bool> {
        Ok(self.board().await?.is_editable(user))
    }

    /// Returns true if the user can move the list
    ///
    /// Only the board owner can move the list
    pub async fn is_movable(&self, user: &User<'_>) -> sqlx::Result<bool> {
        Ok(self.board().await?.can_move_list(user))
    }

    /// Returns true if the list is visible to the user
    pub async fn is_visible(&self, user: Option<&User<'_>>) -> sqlx::Result<bool> {
        Ok(self.board().await?.is_visible(user).await)
    }
}
