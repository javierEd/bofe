use lettre::message::header::ContentType;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use lettre::{Message, transport::smtp::authentication::Credentials};

use crate::config::MAILER_CONFIG;
use crate::constants::*;

use bofe_core::models::{Session, User};

pub async fn send_email(to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
    if !MAILER_CONFIG.enable {
        return Ok(());
    }

    let message = Message::builder()
        .from(
            MAILER_CONFIG
                .sender_address
                .parse()
                .expect("Could not parse mailer sender address"),
        )
        .to(to.parse().expect("Could not parse recipient address"))
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .expect("Could not build message");

    let credentials = Credentials::new(
        MAILER_CONFIG.smtp_username.to_owned(),
        MAILER_CONFIG.smtp_password.to_owned(),
    );

    match MAILER_CONFIG.smtp_security.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(&MAILER_CONFIG.smtp_address),
        "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&MAILER_CONFIG.smtp_address),
        _ => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            MAILER_CONFIG.smtp_address.clone(),
        )),
    }
    .expect("Could not get SMTP transport builder")
    .credentials(credentials)
    .build()
    .send(message)
    .await?;

    Ok(())
}

pub async fn send_password_changed_email(user: &User<'_>) -> anyhow::Result<()> {
    let l10n = user.language_code.to_l10n();

    let message = format!(
        "{} @{},

{}.

{}.

{}: {}",
        l10n.text(KEY_TEXT_HELLO),
        user.username,
        l10n.text(KEY_TEXT_YOUR_PASSWORD_HAS_BEEN_CHANGED),
        l10n.text(KEY_TEXT_IF_YOU_RECOGNIZE_THIS_ACTION),
        l10n.text(KEY_TEXT_IF_NOT),
        MAILER_CONFIG.support_email_address
    );

    send_email(&user.email, &l10n.text(KEY_TEXT_PASSWORD_CHANGED), &message).await
}

pub async fn send_welcome_email(user: &User<'_>) -> anyhow::Result<()> {
    let l10n = user.language_code.to_l10n();
    let text_welcome = l10n.text(KEY_TEXT_WELCOME_TO_BOFE);

    let message = format!(
        "{} @{},

        {}

        {}: {}",
        l10n.text(KEY_TEXT_HELLO),
        user.username,
        text_welcome,
        l10n.text(KEY_TEXT_IF_YOU_HAVE_ANY_QUESTIONS),
        MAILER_CONFIG.support_email_address
    );

    send_email(&user.email, &text_welcome, &message).await
}

pub async fn send_new_session_email(session: &Session<'_>) -> anyhow::Result<()> {
    let user = session.user().await?;
    let l10n = user.language_code.to_l10n();

    let message = format!(
        "{} @{},

{}:

{}

{}.

{}: {}",
        l10n.text(KEY_TEXT_HELLO),
        user.username,
        l10n.text(KEY_TEXT_SOMEONE_HAS_STARTED_A_NEW_SESSION_FROM),
        session.location(),
        l10n.text(KEY_TEXT_IF_YOU_RECOGNIZE_THIS_ACTION),
        l10n.text(KEY_TEXT_IF_NOT),
        MAILER_CONFIG.support_email_address,
    );

    send_email(&user.email, &l10n.text(KEY_TEXT_NEW_SESSION_STARTED), &message).await
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
