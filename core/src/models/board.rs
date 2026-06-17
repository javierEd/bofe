use std::borrow::Cow;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands;
use crate::enums::BoardVisibility;
use crate::models::Activity;

use super::User;

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
    /// Returns true if the user can create cards
    ///
    /// Only members of the board can create cards
    pub async fn can_create_card(&self, user: &User<'_>) -> bool {
        self.is_member(user).await
    }

    /// Returns true if the user can create labels on the board
    ///
    /// Only the board owner can create labels
    pub fn can_create_label(&self, user: &User<'_>) -> bool {
        self.is_editable(user)
    }

    /// Returns true if the user can create lists on the board
    ///
    /// Only the board owner can create lists
    pub fn can_create_list(&self, user: &User<'_>) -> bool {
        self.is_editable(user)
    }

    /// Returns true if the user can create members on the board
    ///
    /// Only the board owner can create members
    pub fn can_create_member(&self, user: &User<'_>) -> bool {
        self.is_editable(user)
    }

    /// Returns true if the user can move cards on the board
    ///
    /// Only the board owner or admin members can move cards
    pub async fn can_move_card(&self, user: &User<'_>) -> bool {
        self.is_admin(user).await
    }

    /// Returns true if the user can move lists on the board
    ///
    /// Only the board owner can move lists
    pub fn can_move_list(&self, user: &User<'_>) -> bool {
        self.is_editable(user)
    }

    /// Returns true if the user is the owner or an admin member of the board
    pub async fn is_admin(&self, user: &User<'_>) -> bool {
        self.user_id == user.id || commands::get_admin_member(self, user).await.is_ok()
    }

    /// Returns true if the user can edit the board
    ///
    /// Only board owner can edit the board
    pub fn is_editable(&self, user: &User<'_>) -> bool {
        self.user_id == user.id
    }

    /// Returns true if the user is the owner or a member of the board
    pub async fn is_member(&self, target_user: &User<'_>) -> bool {
        self.user_id == target_user.id || commands::get_member(self, target_user).await.is_ok()
    }

    /// Returns true if the board is visible to the user
    pub async fn is_visible(&self, target_user: Option<&User<'_>>) -> bool {
        match self.visibility {
            BoardVisibility::Public => true,
            BoardVisibility::Users => target_user.is_some(),
            BoardVisibility::Private => {
                if let Some(user) = target_user {
                    self.is_member(user).await
                } else {
                    false
                }
            }
        }
    }

    pub async fn user(&self) -> sqlx::Result<User<'_>> {
        commands::get_user_by_id(self.user_id).await
    }

    pub async fn last_activity(&self) -> sqlx::Result<Activity> {
        commands::get_last_activity_by_board(self).await
    }
}
