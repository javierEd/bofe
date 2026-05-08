use toolbox::config::MAILER_CONFIG;
use toolbox::mailer::send_email;

use boards_core::models::{Session, User};

pub async fn send_welcome_email(user: &User<'_>) -> anyhow::Result<()> {
    let message = format!(
        "Hello @{},

        Welcome to Boards.

        If you have any questions, please contact us at the following email address: {}",
        user.username, MAILER_CONFIG.support_email_address
    );

    send_email(&user.email, "Welcome to Boards", &message).await
}

pub async fn send_new_session_email(session: &Session<'_>) -> anyhow::Result<()> {
    let user = session.user().await?;

    let message = format!(
        "Hello @{},

Someone has started a new session from:

Location: {}

If you recognize this action, you can ignore this message.

If not, please contact us at the following email address: {}",
        user.username,
        session.location(),
        MAILER_CONFIG.support_email_address,
    );

    send_email(&user.email, "New session started", &message).await
}

pub mod admin_emails {
    use super::*;

    pub async fn send_new_user_email(user: &User<'_>) -> anyhow::Result<()> {
        let message = format!(
            "Hello,

Someone has created a new user account with the following username: @{}",
            user.username
        );

        send_email(
            &MAILER_CONFIG.support_email_address,
            "(Admin) New user account created",
            &message,
        )
        .await
    }
}
