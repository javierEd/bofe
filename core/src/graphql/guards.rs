use async_graphql::{Context, Guard, Result};

use super::CustomContext;

#[derive(Default)]
pub struct UserGuard;

impl Guard for UserGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.user_opt().is_some() {
            Ok(())
        } else {
            Err("Unauthorized".into())
        }
    }
}
