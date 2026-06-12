use validator::{Validate, ValidationErrors};

use crate::constants::{ERROR_IS_INVALID, ERROR_PASSWORD_MUST_CHANGE};
use crate::enums::{ConfirmationAction, CountryCode, LanguageCode};
use crate::models::{Confirmation, User};
use crate::params::{ConfirmationParams, ResetPasswordParams, UpdatePasswordParams};
use crate::{db_pool, jobs_storage};

use super::*;

pub async fn confirm_user_password_reset(confirmation_params: ConfirmationParams) -> ValidationResult {
    finish_confirmation(
        confirmation_params,
        ConfirmationAction::PasswordReset,
        async move |confirmation| {
            let db_pool = db_pool().await;
            let user = confirmation.user().await;
            let new_password = random_string(12..=16);

            sqlx::query!(
                "UPDATE users SET encrypted_password = $2 WHERE disabled_at IS NULL AND id = $1",
                user.id,                         // $1
                encrypt_password(&new_password), // $2
            )
            .execute(db_pool)
            .await
            .or_validation_errors()?;

            jobs_storage()
                .await
                .push_password_changed(&user, Some(new_password))
                .await;

            remove_user_cache(&user).await;

            Ok(())
        },
    )
    .await
}

pub async fn send_user_password_reset_confirmation<'a>(params: ResetPasswordParams) -> sqlx::Result<Confirmation<'a>> {
    let user = get_user_by_username_or_email(&params.username_or_email).await?;

    insert_confirmation(&user, ConfirmationAction::PasswordReset).await
}

pub async fn update_user_password<'a>(user: &User<'_>, params: UpdatePasswordParams) -> ValidationResult<User<'a>> {
    params.validate()?;

    let mut validation_errors = ValidationErrors::new();

    if !user.verify_password(&params.current_password) {
        validation_errors.add("current_password", ERROR_IS_INVALID.clone());

        return Err(validation_errors);
    }

    if params.current_password == params.new_password {
        validation_errors.add("new_password", ERROR_PASSWORD_MUST_CHANGE.clone());

        return Err(validation_errors);
    }

    let db_pool = db_pool().await;
    let encrypted_password = encrypt_password(&params.new_password);

    let updated_user = sqlx::query_as!(
        User,
        r#"UPDATE users SET encrypted_password = $2 WHERE disabled_at IS NULL AND id = $1
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
        user.id,            // $1
        encrypted_password, // $2
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    jobs_storage().await.push_password_changed(user, None).await;

    remove_user_cache(user).await;

    Ok(updated_user)
}
