use apalis_redis::RedisStorage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[cfg(feature = "graphql")]
use apalis::prelude::TaskSink;

use crate::config::MONITOR_CONFIG;
use crate::enums::ActivityAction;

#[cfg(feature = "graphql")]
use crate::models::{Board, Card, Confirmation, List, Session, User};

pub struct JobsStorage {
    pub new_confirmation: RedisStorage<NewConfirmationJob>,
    pub new_session: RedisStorage<NewSessionJob>,
    pub new_user: RedisStorage<NewUserJob>,
    pub password_changed: RedisStorage<PasswordChangedJob>,
    pub activity: RedisStorage<ActivityJob>,
}

impl JobsStorage {
    pub(crate) async fn new() -> Self {
        Self {
            new_confirmation: Self::storage().await,
            new_session: Self::storage().await,
            new_user: Self::storage().await,
            password_changed: Self::storage().await,
            activity: Self::storage().await,
        }
    }

    async fn storage<T: Serialize + for<'de> Deserialize<'de>>() -> RedisStorage<T> {
        let conn = apalis_redis::connect(MONITOR_CONFIG.redis_url.clone())
            .await
            .expect("Could not connect to Redis Jobs DB");

        RedisStorage::new(conn)
    }

    #[cfg(feature = "graphql")]
    pub(crate) async fn push_new_confirmation(&self, confirmation: &Confirmation<'_>, code: &str) {
        self.new_confirmation
            .clone()
            .push(NewConfirmationJob {
                confirmation_id: confirmation.id,
                code: code.to_owned(),
            })
            .await
            .expect("Could not store job");
    }

    #[cfg(feature = "graphql")]
    pub(crate) async fn push_new_session(&self, session: &Session<'_>) {
        self.new_session
            .clone()
            .push(NewSessionJob { session_id: session.id })
            .await
            .expect("Could not store job");
    }

    #[cfg(feature = "graphql")]
    pub(crate) async fn push_new_user(&self, user: &User<'_>) {
        self.new_user
            .clone()
            .push(NewUserJob { user_id: user.id })
            .await
            .expect("Could not store job");
    }

    #[cfg(feature = "graphql")]
    pub(crate) async fn push_password_changed(&self, user: &User<'_>, new_password: Option<String>) {
        self.password_changed
            .clone()
            .push(PasswordChangedJob {
                user_id: user.id,
                new_password,
            })
            .await
            .expect("Could not store job");
    }

    #[cfg(feature = "graphql")]
    pub(crate) async fn push_activity<T, D>(
        &self,
        user: &User<'_>,
        board: &Board<'_>,
        action: ActivityAction,
        target: &T,
        data: &D,
    ) where
        ActivityJob: ActivityJobExt<T, D>,
        D: Serialize,
    {
        self.activity
            .clone()
            .push(ActivityJob::new(user, board, action, target, data))
            .await
            .expect("Could not store job");
    }
}

#[derive(Deserialize, Serialize)]
pub struct NewConfirmationJob {
    pub confirmation_id: Uuid,
    pub code: String,
}

#[derive(Deserialize, Serialize)]
pub struct NewSessionJob {
    pub session_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct NewUserJob {
    pub user_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct PasswordChangedJob {
    pub user_id: Uuid,
    pub new_password: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ActivityJob {
    pub user_id: Uuid,
    pub board_id: Uuid,
    pub action: ActivityAction,
    pub target_id: Uuid,
    pub data: Option<Value>,
}

#[cfg(feature = "graphql")]
pub(crate) trait ActivityJobExt<T, D = ()> {
    fn new(user: &User<'_>, board: &Board<'_>, action: ActivityAction, target: &T, data: &D) -> Self
    where
        D: Serialize;
}

#[cfg(feature = "graphql")]
impl<D> ActivityJobExt<Board<'_>, D> for ActivityJob {
    fn new(user: &User, board: &Board, action: ActivityAction, target: &Board, data: &D) -> Self
    where
        D: Serialize,
    {
        Self {
            user_id: user.id,
            board_id: board.id,
            action,
            target_id: target.id,
            data: serde_json::to_value(data).ok(),
        }
    }
}

#[cfg(feature = "graphql")]
impl<D> ActivityJobExt<Card<'_>, D> for ActivityJob {
    fn new(user: &User, board: &Board, action: ActivityAction, target: &Card, data: &D) -> Self
    where
        D: Serialize,
    {
        Self {
            user_id: user.id,
            board_id: board.id,
            action,
            target_id: target.id,
            data: serde_json::to_value(data).ok(),
        }
    }
}

#[cfg(feature = "graphql")]
impl<D> ActivityJobExt<List<'_>, D> for ActivityJob {
    fn new(user: &User, board: &Board, action: ActivityAction, target: &List, data: &D) -> Self
    where
        D: Serialize,
    {
        Self {
            user_id: user.id,
            board_id: board.id,
            action,
            target_id: target.id,
            data: serde_json::to_value(data).ok(),
        }
    }
}
