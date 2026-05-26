use async_graphql::{Context, ID, Result, Subscription};
use futures_util::{Stream, StreamExt};

use crate::commands;
use crate::graphql::objects::BoardObject;

use super::{CustomContext, IDExt};

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn board(&self, ctx: &Context<'_>, id: ID) -> Result<impl Stream<Item = Option<BoardObject<'_>>>> {
        let id = id.try_into_uuid()?;
        let user = ctx.user_opt();

        let board = commands::get_visible_board_by_id(id, user).await?;

        let mut pubsub = commands::create_or_join_board_channel(&board).await?;

        Ok(async_stream::stream! {
            yield Some(BoardObject(board));

            let mut on_message = pubsub.on_message();

            while on_message.next().await.is_some() {
                yield commands::get_visible_board_by_id(id, user)
                       .await
                       .map(BoardObject).ok();
            }
        })
    }
}
