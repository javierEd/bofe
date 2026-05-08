use chrono::{NaiveDate, Utc};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use toolbox::constants::{ERROR_ALREADY_EXISTS, ERROR_IS_INVALID};

use crate::commands;
use crate::constants::{REGEX_SLUG, REGEX_USERNAME};
use crate::enums::BoardVisibility;

fn validate_birthdate(value: &NaiveDate) -> Result<(), ValidationError> {
    if *value > Utc::now().date_naive() {
        return Err(ERROR_IS_INVALID.clone());
    }

    Ok(())
}

fn validate_country_code(value: &str) -> Result<(), ValidationError> {
    use rust_iso3166::ALL_ALPHA2;

    if !ALL_ALPHA2.contains(&value) {
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

fn validate_slug(value: &str) -> Result<(), ValidationError> {
    if Uuid::try_parse(value).is_ok() {
        return Err(ValidationError::new("Is invalid"));
    }

    Ok(())
}

fn validate_username(value: &str) -> Result<(), ValidationError> {
    if Uuid::try_parse(value).is_ok() {
        return Err(ERROR_IS_INVALID.clone());
    }

    if crate::block_on(commands::user_username_exists(value)) {
        return Err(ERROR_ALREADY_EXISTS.clone());
    }

    Ok(())
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub struct BoardParams {
    #[validate(length(min = 1, max = 255, message = "Can't be blank"))]
    pub name: String,
    #[validate(
        length(min = 1, max = 255, message = "Can't be blank"),
        regex(path = *REGEX_SLUG, message = "Is invalid"),
        custom(function = "validate_slug")
    )]
    pub slug: String,
    pub description: String,
    pub visibility: BoardVisibility,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub struct CardParams {
    pub list_id: Uuid,
    #[validate(length(min = 1, max = 1024, message = "Can't be blank"))]
    pub content: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub struct ListParams {
    pub board_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Can't be blank"))]
    pub name: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub struct UpdateListParams {
    #[validate(length(min = 1, max = 255, message = "Can't be blank"))]
    pub name: String,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::InputObject))]
#[derive(Validate)]
pub struct UserParams {
    #[validate(
        length(min = 3, max = 16, message = "Must have at least 3 characters"),
        regex(path = *REGEX_USERNAME, message = "Is invalid"),
        custom(function = "validate_username")
    )]
    pub username: String,
    #[validate(
        length(min = 5, max = 255, message = "Must have at least 5 characters"),
        email(message = "Is invalid"),
        custom(function = "validate_email")
    )]
    pub email: String,
    #[validate(length(min = 6, max = 128, message = "Must have at least 6 characters"))]
    pub password: String,
    #[validate(length(min = 2, max = 255, message = "Must have at least 2 characters"))]
    pub full_name: String,
    #[validate(required(message = "Can't be blank"), custom(function = "validate_birthdate"))]
    pub birthdate: Option<NaiveDate>,
    #[validate(custom(function = "validate_country_code"))]
    pub country_code: String,
}
