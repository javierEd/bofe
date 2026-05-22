use redis::aio::PubSub;
use redis::{RedisResult, TypedCommands};

use crate::models::Board;
use crate::pubsub_client;

pub async fn create_or_join_board_channel(board: &Board<'_>) -> RedisResult<PubSub> {
    let client = pubsub_client();
    let mut pubsub = client.get_async_pubsub().await?;

    pubsub.subscribe(format!("board.{}", board.id)).await?;

    Ok(pubsub)
}

pub fn notify_board_channel(board: &Board<'_>) -> RedisResult<()> {
    let client = pubsub_client();
    let mut conn = client.get_connection()?;

    conn.publish(format!("board.{}", board.id), "UPDATED")?;

    Ok(())
}
