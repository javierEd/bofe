use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::constants::REGEX_SLUG;
use crate::enums::BoardVisibility;

fn validate_slug(value: &str) -> Result<(), ValidationError> {
    if Uuid::try_parse(value).is_ok() {
        return Err(ValidationError::new("Is invalid"));
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
