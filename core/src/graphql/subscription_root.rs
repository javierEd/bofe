use async_graphql::{Context, ID, Result, Subscription};
use futures_util::{Stream, StreamExt};

use crate::commands;
use crate::graphql::context::CustomExt;
use crate::graphql::objects::{ActivityObject, BoardObject};

use super::IDExt;

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

    async fn board_activities(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        after_id: Option<ID>,
    ) -> Result<impl Stream<Item = Vec<ActivityObject>>> {
        let board_id = board_id.try_into_uuid()?;
        let after_id = after_id.and_then(|id| id.try_into_uuid().ok());
        let user = ctx.user_opt();

        let board = commands::get_visible_board_by_id(board_id, user).await?;

        let mut after_activity = if let Some(id) = after_id {
            Some(commands::get_activity_by_id(id).await?)
        } else {
            None
        };

        let mut pubsub = commands::create_or_join_board_activities_channel(&board).await?;

        Ok(async_stream::stream! {
            let mut on_message = pubsub.on_message();

            loop {
                let activities = commands::get_all_activities_by_board(&board, after_activity.as_ref()).await
                    .unwrap_or_default();

                yield activities.iter().map(|activity| ActivityObject(activity.clone())).collect();

                 if !activities.is_empty() {
                     after_activity = activities.last().cloned();
                 }

                if on_message.next().await.is_none() {
                    break;
                }
            }
        })
    }
}
