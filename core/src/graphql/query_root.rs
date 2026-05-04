use async_graphql::{Context, Object};

use crate::Info;
use crate::graphql::CustomContext;
use crate::graphql::objects::{InfoObject, UserObject};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn current_user(&self, ctx: &Context<'_>) -> Option<UserObject> {
        ctx.user_opt().map(|user| UserObject(user.clone()))
    }

    async fn info(&self) -> InfoObject {
        InfoObject(Info::default())
    }
}
