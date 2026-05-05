use async_graphql::dynamic::indexmap::IndexMap;
use async_graphql::{Context, Error, ErrorExtensions, Name, Object, Result, Value};
use validator::ValidationErrors;

use crate::commands;
use crate::graphql::CustomContext;
use crate::graphql::guards::UserGuard;
use crate::graphql::objects::BoardObject;
use crate::params::BoardParams;

pub struct MutationRoot;

fn to_mutation_error(message: &str, errors: ValidationErrors) -> Error {
    let params_err = Value::Object(
        errors
            .field_errors()
            .iter()
            .map(|(field, error)| {
                let mut details = IndexMap::new();

                details.insert(Name::new("code"), Value::from(error[0].code.clone()));
                details.insert(Name::new("message"), Value::from(error[0].message.clone()));

                (Name::new(field), Value::from(details))
            })
            .collect(),
    );

    Error::new(message).extend_with(|_, e| e.set("params", params_err))
}

#[Object]
impl MutationRoot {
    #[graphql(guard = "UserGuard")]
    async fn create_board(&self, ctx: &Context<'_>, params: BoardParams) -> Result<BoardObject<'_>> {
        let user = ctx.user();

        commands::insert_board(user, params)
            .await
            .map(BoardObject)
            .map_err(|errors| to_mutation_error("Failed to create board", errors))
    }
}
