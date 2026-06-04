use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::graphql::CustomContext;
use crate::models::Label;

pub struct LabelObject<'a>(pub Label<'a>);

#[Object]
impl LabelObject<'_> {
    async fn id(&self) -> ID {
        self.0.id.into()
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn color_code(&self) -> &str {
        &self.0.color_code
    }

    async fn is_editable(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(user) = ctx.user_opt()
            && self.0.is_editable(user).await?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.0.updated_at
    }
}
