use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use toolbox::identity_client::{IdentityClient, IdentityUser};

use crate::commands;

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub identity_user_id: Uuid,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl User {
    pub async fn identity_user(&self, client: &IdentityClient) -> anyhow::Result<IdentityUser<'_>> {
        commands::get_identity_user(client, &self.identity_user_id.to_string()).await
    }
}
