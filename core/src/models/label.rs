use std::borrow::Cow;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands;
use crate::scalars::ColorCode;

use super::{Board, User};

#[derive(Clone, Deserialize, Serialize)]
pub struct Label<'a> {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub name: Cow<'a, str>,
    pub color_code: ColorCode,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Label<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Label<'_> {
    pub async fn board<'a>(&self) -> sqlx::Result<Board<'a>> {
        commands::get_board_by_id(self.board_id).await
    }

    /// Returns true if the user can edit the label
    ///
    /// Only the board owner can edit the label
    pub async fn is_editable(&self, user: &User<'_>) -> sqlx::Result<bool> {
        Ok(self.board().await?.is_editable(user))
    }

    /// Returns true if the label is visible to the user
    pub async fn is_visible(&self, user: Option<&User<'_>>) -> sqlx::Result<bool> {
        Ok(self.board().await?.is_visible(user).await)
    }
}
