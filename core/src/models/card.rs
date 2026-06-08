use std::borrow::Cow;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::commands;
use crate::models::{Board, Label, List, User};

pub struct Card<'a> {
    pub id: Uuid,
    pub list_id: Uuid,
    pub user_id: Uuid,
    pub content: Cow<'a, str>,
    pub position: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Card<'_> {
    pub async fn board<'a>(&self) -> sqlx::Result<Board<'a>> {
        self.list().await?.board().await
    }
    pub async fn list(&self) -> sqlx::Result<List<'_>> {
        commands::get_list_by_id(self.list_id).await
    }

    pub async fn user(&self) -> sqlx::Result<User<'_>> {
        commands::get_user_by_id(self.user_id).await
    }

    pub async fn all_labels(&self) -> sqlx::Result<Vec<Label<'_>>> {
        futures::future::try_join_all(
            commands::get_all_card_labels(self)
                .await?
                .iter()
                .map(|card_label| card_label.label()),
        )
        .await
    }

    pub fn content(&self, max_length: Option<u16>, strip_markdown: Option<bool>) -> String {
        if max_length.is_none() && strip_markdown.is_none() {
            return self.content.to_string();
        }

        let content = if strip_markdown.unwrap_or(false) {
            commands::markdown_to_text(&self.content)
        } else {
            self.content.to_string()
        };

        if let Some(max_length) = max_length
            && (max_length as usize) < content.len()
        {
            content[..max_length as usize].to_owned()
        } else {
            content
        }
    }

    /// Returns true if the user can edit the card
    ///
    /// Only the owner of the card can edit the card
    pub fn is_editable(&self, user: &User) -> bool {
        self.user_id == user.id
    }

    /// Returns true if the user can move the card
    ///
    /// Only the board owner or admin members can move the card
    pub async fn is_movable(&self, user: &User<'_>) -> sqlx::Result<bool> {
        self.list().await?.can_move_card(user).await
    }

    /// Returns true if the card is visible to the user
    pub async fn is_visible(&self, user: Option<&User<'_>>) -> sqlx::Result<bool> {
        self.list().await?.is_visible(user).await
    }
}
