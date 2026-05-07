use std::borrow::Cow;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use toolbox::identity_client::{IdentityClient, IdentityUser};

use crate::commands;
use crate::enums::BoardVisibility;

#[derive(Clone, Deserialize, Serialize)]
pub struct Board<'a> {
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

    pub async fn user(&self) -> sqlx::Result<User> {
        commands::get_user_by_id(self.user_id).await
    }
}

pub struct Card<'a> {
    pub id: Uuid,
    pub list_id: Uuid,
    pub content: Cow<'a, str>,
    pub position: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Card<'_> {
    pub async fn list(&self) -> sqlx::Result<List<'_>> {
        commands::get_list_by_id(self.list_id).await
    }
}

pub struct List<'a> {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: Cow<'a, str>,
    pub position: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl List<'_> {
    pub async fn board(&self) -> sqlx::Result<Board<'_>> {
        commands::get_board_by_id(self.board_id).await
    }

    pub async fn is_visible(&self, target_user: Option<&User>) -> sqlx::Result<bool> {
        let board = self.board().await?;

        Ok(Some(board.user_id) == target_user.map(|u| u.id)
            || (board.visibility == BoardVisibility::Users && target_user.is_some())
            || board.visibility == BoardVisibility::Public)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub identity_user_id: Uuid,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl User {
    pub async fn identity_user(&self, client: &IdentityClient) -> anyhow::Result<IdentityUser<'_>> {
        commands::get_identity_user(client, &self.identity_user_id.to_string()).await
    }
}
