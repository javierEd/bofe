use chrono::{NaiveDate, Utc};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::commands;
use crate::constants::*;
use crate::enums::{BoardVisibility, CountryCode};

fn validate_birthdate(value: &NaiveDate) -> Result<(), ValidationError> {
    if *value > Utc::now().date_naive() {
        return Err(ERROR_IS_INVALID.clone());
    }

    Ok(())
}

fn validate_email(value: &str) -> Result<(), ValidationError> {
    if crate::block_on(commands::user_email_exists(value)) {
        return Err(ERROR_ALREADY_EXISTS.clone());
    }

    Ok(())
}

fn validate_expires_at(value: &NaiveDate) -> Result<(), ValidationError> {
    if *value <= Utc::now().date_naive() {
        return Err(ERROR_IS_INVALID.clone());
    }

    Ok(())
}

fn validate_presence(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ERROR_CANT_BE_BLANK.clone());
    }

    Ok(())
}

fn validate_slug(value: &str) -> Result<(), ValidationError> {
    if Uuid::try_parse(value).is_ok() {
        return Err(ValidationError::new("Is invalid"));
    }

    Ok(())
}

fn validate_username(value: &str) -> Result<(), ValidationError> {
    if Uuid::try_parse(value).is_ok() || BLACKLISTED_USERNAMES.contains(&value.to_lowercase().as_str()) {
        return Err(ERROR_IS_INVALID.clone());
    }

    if crate::block_on(commands::user_username_exists(value)) {
        return Err(ERROR_ALREADY_EXISTS.clone());
    }

    Ok(())
}

#[derive(Validate)]
pub struct ApplicationParams {
    #[validate(
        custom(function = "validate_presence"),
        length(max = 255, message = "Must have at most 255 characters")
    )]
    pub name: String,
    #[validate(custom(function = "validate_expires_at"))]
    pub expires_at: Option<NaiveDate>,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub(crate) struct BoardParams {
    #[validate(
        custom(function = "validate_presence"),
        length(max = 255, message = "Must have at most 255 characters")
    )]
    pub name: String,
    #[validate(
        length(min = 1, max = 255, message = "Must be between 1 and 255 characters"),
        regex(path = *REGEX_SLUG, message = "Is invalid"),
        custom(function = "validate_slug")
    )]
    pub slug: String,
    pub description: String,
    pub visibility: BoardVisibility,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub(crate) struct CardParams {
    pub list_id: Uuid,
    #[validate(
        custom(function = "validate_presence"),
        length(max = 1024, message = "Must have at most 1024 characters")
    )]
    pub content: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub(crate) struct LabelParams {
    pub board_id: Uuid,
    #[validate(
        custom(function = "validate_presence"),
        length(max = 255, message = "Must have at most 255 characters")
    )]
    pub name: String,
    #[validate(
        length(min = 4, max = 7, message = "Must have at least 4 characters"),
        regex(path = *REGEX_COLOR_CODE, message = "Is invalid")
    )]
    pub color_code: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub(crate) struct ListParams {
    pub board_id: Uuid,
    #[validate(
        custom(function = "validate_presence"),
        length(max = 255, message = "Must have at most 255 characters")
    )]
    pub name: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
pub(crate) struct MemberParams {
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub is_admin: bool,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub(crate) struct SessionParams {
    #[validate(length(min = 1, max = 255, message = "Must have between 1 and 255 characters"))]
    pub username_or_email: String,
    #[validate(length(min = 1, max = 255, message = "Must have between 1 and 255 characters"))]
    pub password: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub(crate) struct UpdateLabelParams {
    #[validate(
        custom(function = "validate_presence"),
        length(max = 255, message = "Must have at most 255 characters")
    )]
    pub name: String,
    #[validate(
        length(min = 4, max = 7, message = "Must have at least 4 characters"),
        regex(path = *REGEX_COLOR_CODE, message = "Is invalid")
    )]
    pub color_code: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub(crate) struct UpdateListParams {
    #[validate(
        custom(function = "validate_presence"),
        length(max = 255, message = "Must have at most 255 characters")
    )]
    pub name: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
pub(crate) struct UpdateMemberParams {
    pub is_admin: bool,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub(crate) struct UpdatePasswordParams {
    #[validate(length(min = 1, max = 255, message = "Must have between 1 and 255 characters"))]
    pub current_password: String,
    #[validate(length(min = 6, max = 255, message = "Must have between 6 and 255 characters"))]
    pub new_password: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub struct UpdateProfileParams {
    #[validate(
        custom(function = "validate_presence"),
        length(max = 255, message = "Must have at most 255 characters")
    )]
    pub display_name: String,
    #[validate(
        custom(function = "validate_presence"),
        length(max = 255, message = "Must have at most 255 characters")
    )]
    pub full_name: String,
    #[validate(required(message = "Can't be blank"), custom(function = "validate_birthdate"))]
    pub birthdate: Option<NaiveDate>,
    pub country_code: CountryCode,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub(crate) struct UserParams {
    #[validate(
        length(min = 3, max = 16, message = "Must have between 3 and 16 characters"),
        regex(path = *REGEX_USERNAME, message = "Is invalid"),
        custom(function = "validate_username")
    )]
    pub username: String,
    #[validate(
        length(min = 5, max = 255, message = "Must have between 5 and 255 characters"),
        email(message = "Is invalid"),
        custom(function = "validate_email")
    )]
    pub email: String,
    #[validate(length(min = 6, max = 255, message = "Must have between 6 and 255 characters"))]
    pub password: String,
    #[validate(
        custom(function = "validate_presence"),
        length(max = 255, message = "Must have at most 255 characters")
    )]
    pub full_name: String,
    #[validate(required(message = "Can't be blank"), custom(function = "validate_birthdate"))]
    pub birthdate: Option<NaiveDate>,
    pub country_code: CountryCode,
}
