use apalis::prelude::TaskSink;
use apalis_redis::RedisStorage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::MONITOR_CONFIG,
    models::{Confirmation, Session, User},
};

pub struct JobsStorage {
    pub new_confirmation: RedisStorage<NewConfirmationJob>,
    pub new_session: RedisStorage<NewSessionJob>,
    pub new_user: RedisStorage<NewUserJob>,
    pub password_changed: RedisStorage<PasswordChangedJob>,
}

impl JobsStorage {
    pub(crate) async fn new() -> Self {
        Self {
            new_confirmation: Self::storage().await,
            new_session: Self::storage().await,
            new_user: Self::storage().await,
            password_changed: Self::storage().await,
        }
    }

    async fn storage<T: Serialize + for<'de> Deserialize<'de>>() -> RedisStorage<T> {
        let conn = apalis_redis::connect(MONITOR_CONFIG.redis_url.clone())
            .await
            .expect("Could not connect to Redis Jobs DB");

        RedisStorage::new(conn)
    }

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

    pub(crate) async fn push_new_session(&self, session: &Session<'_>) {
        self.new_session
            .clone()
            .push(NewSessionJob { session_id: session.id })
            .await
            .expect("Could not store job");
    }

    pub(crate) async fn push_new_user(&self, user: &User<'_>) {
        self.new_user
            .clone()
            .push(NewUserJob { user_id: user.id })
            .await
            .expect("Could not store job");
    }

    pub(crate) async fn push_password_changed(&self, user: &User<'_>) {
        self.password_changed
            .clone()
            .push(PasswordChangedJob { user_id: user.id })
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
}
