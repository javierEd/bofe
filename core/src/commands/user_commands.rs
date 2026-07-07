use cached::AsyncRedisCache;
use cached::macros::concurrent_cached;
use uuid::Uuid;

#[cfg(feature = "graphql")]
use validator::Validate;

use crate::constants::*;
use crate::db_pool;
use crate::enums::{CountryCode, LanguageCode};
use crate::models::User;
use crate::pagination::{CursorPage, CursorParams};

#[cfg(feature = "graphql")]
use crate::jobs_storage;
#[cfg(feature = "graphql")]
use crate::params::{UpdateProfileParams, UserParams};

use super::*;

#[cfg(feature = "graphql")]
pub(crate) async fn authenticate_user<'a>(username_or_email: &str, password: &str) -> sqlx::Result<User<'a>> {
    let user = get_user_by_username_or_email(username_or_email).await?;

    if user.verify_password(password) {
        Ok(user)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

#[cfg(feature = "graphql")]
pub(crate) async fn delete_user(user: &User<'_>) -> sqlx::Result<bool> {
    if has_boards(user).await {
        return Err(sqlx::Error::InvalidArgument("Cannot have boards".to_owned()));
    }

    let db_pool = db_pool().await;

    let _ = finish_all_sessions(user).await;

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(db_pool)
        .await?;

    remove_user_cache(user).await;

    Ok(true)
}

#[cfg(feature = "graphql")]
pub fn get_user_avatar_image(user: &User<'_>, size: u16) -> anyhow::Result<Vec<u8>> {
    if !(32..=512).contains(&size) || size & (size - 1) != 0 {
        return Err(anyhow::anyhow!("Invalid avatar image size"));
    }

    let avatar_image_path = user.avatar_image_path(size);

    if !avatar_image_path.exists() {
        let avatar_image = text_icon(&user.username, size).expect("Could not create text icon");

        std::fs::create_dir_all(avatar_image_path.parent().expect("Could not create storage dir"))?;

        avatar_image
            .save(&avatar_image_path)
            .expect("Could not save avatar image");
    }

    Ok(std::fs::read(&avatar_image_path).expect("Could not read avatar image"))
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    ty = "AsyncRedisCache<Uuid, User>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_BY_ID).await }"##
)]
pub async fn get_user_by_id<'a>(id: Uuid) -> sqlx::Result<User<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        r#"SELECT
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
            updated_at
        FROM users
        WHERE disabled_at IS NULL AND id = $1 LIMIT 1"#,
        id
    )
    .fetch_one(db_pool)
    .await
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ username.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, User>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_BY_USERNAME).await }"##
)]
pub(crate) async fn get_user_by_username<'a>(username: &str) -> sqlx::Result<User<'a>> {
    if username.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        r#"SELECT
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
            updated_at
        FROM users WHERE disabled_at IS NULL AND LOWER(username) = LOWER($1) LIMIT 1"#,
        username
    )
    .fetch_one(db_pool)
    .await
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ username_or_email.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, User>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_BY_USERNAME_OR_EMAIL).await }"##
)]
pub(crate) async fn get_user_by_username_or_email<'a>(username_or_email: &str) -> sqlx::Result<User<'a>> {
    if username_or_email.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    let db_pool = db_pool().await;

    sqlx::query_as!(
        User,
        r#"SELECT
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
            updated_at
        FROM users
        WHERE disabled_at IS NULL AND (LOWER(username) = $1 OR (email_confirmed_at IS NOT NULL AND LOWER(email) = $1))
        LIMIT 1"#,
        username_or_email.to_lowercase()
    )
    .fetch_one(db_pool)
    .await
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ email.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, Uuid>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_ID_BY_EMAIL).await }"##
)]
async fn get_user_id_by_email(email: &str) -> sqlx::Result<Uuid> {
    if email.is_empty() {
        return Err(sqlx::Error::InvalidArgument("email".to_owned()));
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        r#"SELECT id AS "id!" FROM users WHERE LOWER(email) = LOWER($1) LIMIT 1"#,
        email // $1
    )
    .fetch_one(db_pool)
    .await
    .map(|record| record.id)
}

#[concurrent_cached(
    map_error = r##"|_| sqlx::Error::RowNotFound"##,
    convert = r#"{ username.to_lowercase() }"#,
    ty = "AsyncRedisCache<String, Uuid>",
    create = r##"{ redis_cache_store(CACHE_PREFIX_GET_USER_ID_BY_USERNAME).await }"##
)]
async fn get_user_id_by_username(username: &str) -> sqlx::Result<Uuid> {
    if username.is_empty() {
        return Err(sqlx::Error::InvalidArgument("username".to_owned()));
    }

    let db_pool = db_pool().await;

    sqlx::query!(
        r#"SELECT id AS "id!" FROM users WHERE LOWER(username) = LOWER($1) LIMIT 1"#,
        username // $1
    )
    .fetch_one(db_pool)
    .await
    .map(|record| record.id)
}

#[cfg(feature = "graphql")]
pub(crate) async fn insert_user<'a>(params: UserParams) -> ValidationResult<User<'a>> {
    params.validate()?;

    let db_pool = db_pool().await;
    let display_name = params.full_name.split(' ').next().unwrap();
    let encrypted_password = encrypt_password(&params.password);

    let user = sqlx::query_as!(
        User,
        r#"INSERT INTO users (
            username,
            email,
            encrypted_password,
            display_name,
            full_name,
            birthdate,
            language_code,
            country_code
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
        params.username,                               // $1
        params.email.to_lowercase(),                   // $2
        encrypted_password,                            // $3
        display_name,                                  // $4
        params.full_name,                              // $5
        params.birthdate,                              // $6
        params.language_code.unwrap_or_default() as _, // $7
        params.country_code as _,                      // $8
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    jobs_storage().await.push_new_user(&user).await;

    Ok(user)
}

pub async fn paginate_users<'a>(cursor_params: CursorParams, query: &str) -> CursorPage<User<'a>> {
    let query = query.trim();

    if query.len() < 3 {
        return CursorPage::default();
    }

    let db_pool = db_pool().await;

    CursorPage::new(
        &cursor_params,
        |node: &User| node.id,
        async |after| get_user_by_id(after).await.ok(),
        async |cursor_resource, limit| {
            let cursor_username = cursor_resource.map(|c| c.username.to_string());

            sqlx::query_as!(
                User,
                r#"SELECT
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
                    updated_at
                FROM users
                WHERE
                    ($1::text IS NULL OR username > $1) AND (username ILIKE $2 OR display_name ILIKE $2)
                ORDER BY username ASC LIMIT $3"#,
                cursor_username,        // $1
                format!("%{}%", query), // $2
                limit,                  // $3
            )
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
        },
    )
    .await
}

#[cfg(feature = "graphql")]
pub(crate) async fn remove_user_cache(user: &User<'_>) {
    let username = user.username.to_lowercase();
    let email = user.email.to_lowercase();

    tokio::join!(
        GET_USER_BY_ID.cache_remove(CACHE_PREFIX_GET_USER_BY_ID, &user.id),
        GET_USER_BY_USERNAME.cache_remove(CACHE_PREFIX_GET_USER_BY_USERNAME, &username),
        GET_USER_BY_USERNAME_OR_EMAIL.cache_remove(CACHE_PREFIX_GET_USER_BY_USERNAME_OR_EMAIL, &username),
        GET_USER_BY_USERNAME_OR_EMAIL.cache_remove(CACHE_PREFIX_GET_USER_BY_USERNAME_OR_EMAIL, &email),
        GET_USER_ID_BY_EMAIL.cache_remove(CACHE_PREFIX_GET_USER_ID_BY_EMAIL, &email),
        GET_USER_ID_BY_USERNAME.cache_remove(CACHE_PREFIX_GET_USER_ID_BY_USERNAME, &username),
    );
}

#[cfg(feature = "graphql")]
pub async fn update_user_profile<'a>(user: &User<'_>, params: UpdateProfileParams) -> ValidationResult<User<'a>> {
    params.validate()?;

    let db_pool = db_pool().await;

    let updated_user = sqlx::query_as!(
        User,
        r#"UPDATE users SET display_name = $2, full_name = $3, birthdate = $4, language_code = $5, country_code = $6
        WHERE disabled_at IS NULL AND id = $1
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
        user.id,                                       // $1
        params.display_name,                           // $2
        params.full_name,                              // $3
        params.birthdate,                              // $4
        params.language_code.unwrap_or_default() as _, // $5
        params.country_code as _                       // $6
    )
    .fetch_one(db_pool)
    .await
    .or_validation_errors()?;

    remove_user_cache(user).await;

    Ok(updated_user)
}

#[cfg(feature = "graphql")]
pub(crate) async fn user_email_exists(email: &str) -> bool {
    get_user_id_by_email(email).await.is_ok()
}

#[cfg(feature = "graphql")]
pub(crate) async fn user_username_exists(username: &str) -> bool {
    get_user_id_by_username(username).await.is_ok()
}

#[cfg(test)]
mod tests {
    use crate::test_utils::insert_test_user;

    use super::*;

    #[tokio::test]
    async fn authenticate_user_with_valid_params_return_ok() {
        let password = fake_password();
        let user = insert_test_user(Some(password)).await;

        let result = authenticate_user(user.username, password).await;

        assert!(result.is_ok());

        let authenticated_user = result.user.unwrap();

        assert_eq!(authenticate_user.id, user.id);
    }

    #[tokio::test]
    async fn authenticate_user_with_invalid_password_return_err() {
        let password = fake_password();
        let user = insert_test_user(None).await;

        let result = authenticate_user(user.username, password).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_user_with_valid_params_return_ok() {
        let user = insert_test_user().await;

        let result = delete_user(&user).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn user_email_exists_with_unused_email_return_true() {
        let email = fake_email();

        let is_unused = user_email_exists(&email).await;

        assert!(is_unused);
    }

    #[tokio::test]
    async fn user_email_exists_with_used_email_return_false() {
        let user = insert_test_user(None).await;

        let is_unused = user_email_exists(&user.email).await;

        assert!(!is_unused);
    }

    #[tokio::test]
    async fn user_username_exists_with_unused_username_return_true() {
        let username = fake_username();

        let is_unused = user_username_exists(&username).await;

        assert!(is_unused);
    }

    #[tokio::test]
    async fn user_username_exists_with_used_username_return_false() {
        let user = insert_test_user().await;

        let is_unused = user_username_exists(&user.username).await;

        assert!(!is_unused);
    }
}
