use std::net::IpAddr;

use async_graphql::{Context, EmptySubscription, Schema, SchemaBuilder};

use crate::models::{Application, User};

mod guards;
mod mutation_root;
mod objects;
mod query_root;

use mutation_root::MutationRoot;
use query_root::QueryRoot;

pub type GraphqlSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub trait GraphqlSchemaExt {
    fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription>;
}

impl GraphqlSchemaExt for GraphqlSchema {
    fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription> {
        Schema::build(QueryRoot, MutationRoot, EmptySubscription)
    }
}

trait CustomContext {
    fn application(&self) -> &Application<'_>;

    fn client_ip(&self) -> &IpAddr;
    fn user(&self) -> &User<'_>;

    fn user_opt(&self) -> Option<&User<'_>>;
}

impl CustomContext for Context<'_> {
    fn application(&self) -> &Application<'_> {
        self.data_unchecked::<Application<'_>>()
    }

    fn client_ip(&self) -> &IpAddr {
        self.data_unchecked::<IpAddr>()
    }

    fn user(&self) -> &User<'_> {
        self.data_unchecked::<User<'_>>()
    }

    fn user_opt(&self) -> Option<&User<'_>> {
        self.data_opt::<User<'_>>()
    }
}
