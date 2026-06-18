use redis::{AsyncTypedCommands, RedisResult};

#[cfg(feature = "graphql")]
use redis::aio::PubSub;

use crate::im_db_client;
use crate::models::Board;

#[cfg(feature = "graphql")]
pub async fn create_or_join_board_activities_channel(board: &Board<'_>) -> RedisResult<PubSub> {
    pubsub_subscribe(&format!("board_activities:{}", board.id)).await
}

#[cfg(feature = "graphql")]
pub async fn create_or_join_board_channel(board: &Board<'_>) -> RedisResult<PubSub> {
    pubsub_subscribe(&format!("board:{}", board.id)).await
}

pub async fn notify_board_activities_channel(board: &Board<'_>) -> RedisResult<usize> {
    pubsub_publish(&format!("board_activities:{}", board.id), "UPDATED").await
}

pub async fn notify_board_channel(board: &Board<'_>) -> RedisResult<usize> {
    pubsub_publish(&format!("board:{}", board.id), "UPDATED").await
}

async fn pubsub_publish(channel: &str, message: &str) -> RedisResult<usize> {
    let client = im_db_client();
    let mut conn = client.get_multiplexed_async_connection().await?;

    conn.publish(channel, message).await
}

#[cfg(feature = "graphql")]
async fn pubsub_subscribe(channel: &str) -> RedisResult<PubSub> {
    let client = im_db_client();
    let mut pubsub = client.get_async_pubsub().await?;

    pubsub.subscribe(channel).await?;

    Ok(pubsub)
}
