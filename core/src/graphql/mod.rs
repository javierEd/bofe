use async_graphql::{Context, EmptyMutation, EmptySubscription, Schema, SchemaBuilder};

use toolbox::identity_client::IdentityClient;

use crate::models::User;

mod guards;
mod objects;
mod query_root;

use query_root::QueryRoot;

pub type GraphqlSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub trait GraphqlSchemaExt {
    fn builder() -> SchemaBuilder<QueryRoot, EmptyMutation, EmptySubscription>;
}

impl GraphqlSchemaExt for GraphqlSchema {
    fn builder() -> SchemaBuilder<QueryRoot, EmptyMutation, EmptySubscription> {
        Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
    }
}

trait CustomContext {
    fn identity_client(&self) -> &IdentityClient;

    #[allow(dead_code)]
    fn user(&self) -> &User;

    fn user_opt(&self) -> Option<&User>;
}

impl CustomContext for Context<'_> {
    fn identity_client(&self) -> &IdentityClient {
        self.data_unchecked::<IdentityClient>()
    }

    fn user(&self) -> &User {
        self.data_unchecked::<User>()
    }

    fn user_opt(&self) -> Option<&User> {
        self.data_opt::<User>()
    }
}
