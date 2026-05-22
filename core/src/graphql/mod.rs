use std::net::IpAddr;

use async_graphql::{Context, ID, Result, Schema, SchemaBuilder};
use uuid::Uuid;

use crate::models::{Application, Session, User};

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

trait CustomContext {
    fn application(&self) -> &Application<'_>;

    fn client_ip(&self) -> &IpAddr;

    fn session(&self) -> &Session<'_>;

    #[allow(dead_code)]
    fn session_opt(&self) -> Option<&Session<'_>>;
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

    fn session(&self) -> &Session<'_> {
        self.data_unchecked::<Session<'_>>()
    }

    fn session_opt(&self) -> Option<&Session<'_>> {
        self.data_opt::<Session<'_>>()
    }

    fn user(&self) -> &User<'_> {
        self.data_unchecked::<User<'_>>()
    }

    fn user_opt(&self) -> Option<&User<'_>> {
        self.data_opt::<User<'_>>()
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
