use std::borrow::Cow;
use std::fmt::Display;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands;
use crate::enums::{BoardVisibility, CountryCode, LanguageCode};

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
    /// Returns true if the user can create cards
    ///
    /// Only members of the board can create cards
    pub async fn can_create_card(&self, user: &User<'_>) -> bool {
        self.is_member(user).await
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
    pub async fn board<'a>(&self) -> sqlx::Result<Board<'a>> {
        self.list().await?.board().await
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

    pub async fn list(&self) -> sqlx::Result<List<'_>> {
        commands::get_list_by_id(self.list_id).await
    }

    pub async fn user(&self) -> sqlx::Result<User<'_>> {
        commands::get_user_by_id(self.user_id).await
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

#[derive(Clone, Deserialize, Serialize)]
pub struct User<'a> {
    pub id: Uuid,
    pub username: Cow<'a, str>,
    pub email: Cow<'a, str>,
    pub(crate) encrypted_password: Cow<'a, str>,
    pub full_name: Cow<'a, str>,
    pub display_name: Cow<'a, str>,
    pub birthdate: NaiveDate,
    pub language_code: LanguageCode,
    pub country_code: CountryCode,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for User<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl User<'_> {
    pub(crate) fn initials(&self) -> String {
        self.username[0..2].to_uppercase()
    }

    pub(crate) fn verify_password(&self, password: &str) -> bool {
        commands::verify_password(&self.encrypted_password, password)
    }
}
