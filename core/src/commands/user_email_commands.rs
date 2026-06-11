use validator::{Validate, ValidationErrors};

use crate::db_pool;
use crate::enums::{ConfirmationAction, CountryCode, LanguageCode};
use crate::models::User;
use crate::params::{ConfirmationParams, UpdateEmailParams};

use super::{OrValidationErrors, ValidationResult, finish_confirmation, get_confirmation_by_id, remove_user_cache};

pub async fn confirm_user_email<'a>(
    user: &User<'_>,
    confirmation_params: ConfirmationParams,
) -> ValidationResult<User<'a>> {
    confirmation_params.validate()?;

    let confirmation = get_confirmation_by_id(confirmation_params.id)
        .await
        .map_err(|_| ValidationErrors::new())?;

    if confirmation.user_id != user.id {
        return Err(ValidationErrors::new());
    }

    finish_confirmation(
        &confirmation,
        ConfirmationAction::Email,
        &confirmation_params.code,
        async move || {
            let db_pool = db_pool().await;

            let updated_user = sqlx::query_as!(
                User,
                r#"UPDATE users SET email_confirmed_at = current_timestamp
                WHERE disabled_at IS NULL AND email_confirmed_at IS NULL AND id = $1
                RETURNING
                    id,
                    username,
                    email,
                    email_confirmed_at,
                    encrypted_password,
                    full_name,
                    display_name,
                    birthdate,
                    language_code AS "language_code!: LanguageCode",
                    country_code AS "country_code!: CountryCode",
                    disabled_at,
                    created_at,
                    updated_at"#,
                user.id, // $1
            )
            .fetch_one(db_pool)
            .await
            .or_validation_errors()?;

            remove_user_cache(user).await;

            Ok(updated_user)
        },
    )
    .await
}

pub async fn update_user_email<'a>(user: &User<'_>, params: UpdateEmailParams) -> ValidationResult<User<'a>> {
    params.validate()?;

    let db_pool = db_pool().await;

    let updated_user = sqlx::query_as!(
        User,
        r#"UPDATE users SET email = $2, email_confirmed_at = NULL WHERE disabled_at IS NULL AND id = $1
        RETURNING
            id,
            username,
            email,
            email_confirmed_at,
            encrypted_password,
            full_name,
            display_name,
            birthdate,
            language_code AS "language_code!: LanguageCode",
            country_code AS "country_code!: CountryCode",
            disabled_at,
            created_at,
            updated_at"#,
        user.id,      // $1
        params.email, // $2
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    remove_user_cache(user).await;

    Ok(updated_user)
}
