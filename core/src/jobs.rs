use serde::{Deserialize, Serialize};
use uuid::Uuid;

use toolbox::identity_client::IdentityClient;

#[derive(Deserialize, Serialize)]
pub struct NewFileJob {
    pub file_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct NewUserJob {
    pub identity_client: IdentityClient,
    pub user_id: Uuid,
}
