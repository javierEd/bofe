use chrono::Utc;
use validator::Validate;

use toolbox::rand::random_string;
use toolbox::validator::{OrValidationErrors, ValidationResult};

use crate::config::APPLICATION_CONFIG;
use crate::db_pool;
use crate::models::Application;
use crate::params::ApplicationParams;

pub async fn insert_application<'a>(params: ApplicationParams) -> ValidationResult<Application<'a>> {
    params.validate()?;

    let db_pool = db_pool().await;

    let name = params.name.trim();
    let token = random_string(APPLICATION_CONFIG.token_length());
    let expires_at = params
        .expires_at
        .map(|date| date.and_time(Utc::now().time()).and_utc())
        .unwrap_or_else(|| Utc::now() + APPLICATION_CONFIG.ttl());

    let access_token = sqlx::query_as!(
        Application,
        "INSERT INTO applications (name, token, expires_at) VALUES ($1, $2, $3) RETURNING *",
        name,       // $1
        token,      // $2
        expires_at, // $3
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    Ok(access_token)
}
