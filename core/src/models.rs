use std::borrow::Cow;
use std::fmt::Display;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands;
use crate::enums::BoardVisibility;

pub struct Application<'a> {
    pub id: Uuid,
    pub name: Cow<'a, str>,
    pub token: Cow<'a, str>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Application<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct Board<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: Cow<'a, str>,
    pub slug: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub visibility: BoardVisibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Board<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Board<'_> {
    pub fn is_editable(&self, user: Option<&User>) -> bool {
        Some(self.user_id) == user.map(|u| u.id)
    }

    pub fn is_visible(&self, target_user: Option<&User>) -> bool {
        Some(self.user_id) == target_user.map(|u| u.id)
            || (self.visibility == BoardVisibility::Users && target_user.is_some())
            || self.visibility == BoardVisibility::Public
    }

    pub async fn user(&self) -> sqlx::Result<User<'_>> {
        commands::get_user_by_id(self.user_id).await
    }
}

pub(crate) struct Card<'a> {
    pub id: Uuid,
    pub list_id: Uuid,
    pub user_id: Uuid,
    pub content: Cow<'a, str>,
    pub position: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Card<'_> {
    #[allow(dead_code)]
    pub fn is_editable(&self, user: Option<&User>) -> bool {
        Some(self.user_id) == user.map(|u| u.id)
    }

    pub async fn list(&self) -> sqlx::Result<List<'_>> {
        commands::get_list_by_id(self.list_id).await
    }
}

pub(crate) struct List<'a> {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub name: Cow<'a, str>,
    pub position: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl List<'_> {
    pub async fn board(&self) -> sqlx::Result<Board<'_>> {
        commands::get_board_by_id(self.board_id).await
    }

    pub fn is_editable(&self, user: Option<&User>) -> bool {
        Some(self.user_id) == user.map(|u| u.id)
    }

    pub async fn is_visible(&self, target_user: Option<&User<'_>>) -> sqlx::Result<bool> {
        if self.is_editable(target_user) {
            return Ok(true);
        }

        let board = self.board().await?;

        Ok(Some(board.user_id) == target_user.map(|u| u.id)
            || (board.visibility == BoardVisibility::Users && target_user.is_some())
            || board.visibility == BoardVisibility::Public)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct User<'a> {
    pub id: Uuid,
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub(crate) encrypted_password: Cow<'a, str>,
    pub full_name: Cow<'a, str>,
    pub display_name: Cow<'a, str>,
    pub birthdate: NaiveDate,
    pub language_code: Cow<'a, str>,
    pub country_code: Cow<'a, str>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for User<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
