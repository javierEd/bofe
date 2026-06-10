use std::borrow::Cow;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands;
use crate::enums::{ConfirmationAction, CountryCode};

mod board;
mod card;
mod label;
mod user;

pub(crate) use board::Board;
pub(crate) use card::Card;
pub(crate) use label::Label;
pub use user::User;

#[derive(Clone, Deserialize, Serialize)]
pub struct Application<'a> {
    pub id: Uuid,
    pub name: Cow<'a, str>,
    pub token: Cow<'a, str>,
    pub expires_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Application<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

pub(crate) struct CardLabel {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    pub card_id: Uuid,
    pub label_id: Uuid,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

impl CardLabel {
    pub async fn label<'a>(&self) -> sqlx::Result<Label<'a>> {
        commands::get_label_by_id(self.label_id).await
    }
}

pub struct Confirmation<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: ConfirmationAction,
    pub(crate) encrypted_code: Cow<'a, str>,
    pub pending_attempts: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Confirmation<'_> {
    pub async fn user(&self) -> User<'_> {
        commands::get_user_by_id(self.user_id)
            .await
            .expect("Could not get user")
    }

    pub fn verify_code(&self, code: &str) -> bool {
        commands::verify_password(&self.encrypted_code, code)
    }
}

pub(crate) struct List<'a> {
    pub id: Uuid,
    pub board_id: Uuid,
    #[allow(dead_code)]
    pub user_id: Uuid,
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

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct Member {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Member {
    pub async fn board<'a>(&self) -> sqlx::Result<Board<'a>> {
        commands::get_board_by_id(self.board_id).await
    }

    pub async fn user<'a>(&self) -> sqlx::Result<User<'a>> {
        commands::get_user_by_id(self.user_id).await
    }

    /// Returns true if the user can edit the member
    ///
    /// Only the owner of the board can edit the member
    pub async fn is_editable(&self, user: &User<'_>) -> sqlx::Result<bool> {
        Ok(self.board().await?.is_editable(user))
    }

    /// Returns true if the user can remove the member
    ///
    /// Only the owner of the board or the same use can remove the member
    pub async fn is_removable(&self, user: &User<'_>) -> sqlx::Result<bool> {
        Ok(self.user_id == user.id || self.is_editable(user).await?)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Session<'a> {
    pub id: Uuid,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub token: Cow<'a, str>,
    pub ip_address: Cow<'a, str>,
    pub country_code: Option<CountryCode>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub refreshed_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Session<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Session<'_> {
    pub fn location(&self) -> String {
        let Some(country) = self.country_code else {
            return "Unknown".to_owned();
        };

        let mut location = country.name().to_owned();

        if let Some(region) = &self.region {
            location += &format!(", {region}");
        }

        if let Some(city) = &self.city {
            location += &format!(", {city}");
        }

        location
    }

    pub async fn user<'a>(&self) -> sqlx::Result<User<'a>> {
        commands::get_user_by_id(self.user_id).await
    }
}
