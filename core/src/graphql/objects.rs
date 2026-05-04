use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};

use toolbox::graphql::objects::IdentityUserObject;

use crate::Info;
use crate::graphql::CustomContext;
use crate::models::User;

pub struct InfoObject(pub Info);

#[Object]
impl InfoObject {
    async fn built_at(&self) -> DateTime<Utc> {
        self.0.built_at
    }

    async fn version(&self) -> &str {
        &self.0.version
    }
}

pub struct UserObject(pub User);

#[Object]
impl UserObject {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn identity_user(&self, ctx: &Context<'_>) -> Result<IdentityUserObject<'_>> {
        Ok(self
            .0
            .identity_user(ctx.identity_client())
            .await
            .map(IdentityUserObject)?)
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
