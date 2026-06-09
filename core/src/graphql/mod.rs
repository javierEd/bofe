use async_graphql::{ID, Result, Schema, SchemaBuilder};
use uuid::Uuid;

mod context;
mod guards;
mod mutation_root;
mod objects;
mod query_root;
mod subscription_root;

use mutation_root::MutationRoot;
use query_root::QueryRoot;
use subscription_root::SubscriptionRoot;

pub type GraphqlSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub trait GraphqlSchemaExt {
    fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, SubscriptionRoot>;
}

impl GraphqlSchemaExt for GraphqlSchema {
    fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, SubscriptionRoot> {
        Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
    }
}

trait IDExt {
    fn try_into_uuid(&self) -> Result<Uuid>;
}

impl IDExt for ID {
    fn try_into_uuid(&self) -> Result<Uuid> {
        Ok(Uuid::try_parse(self.as_ref())?)
    }
}
