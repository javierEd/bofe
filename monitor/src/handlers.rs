use std::net::IpAddr;
use std::str::FromStr;

use apalis::prelude::BoxDynError;

use bofe_core::commands;
use bofe_core::jobs::{ActivityJob, NewConfirmationJob, NewSessionJob, NewUserJob, PasswordChangedJob};

use crate::ip_geo::IpGeo;
use crate::mailer::{admin_emails, send_new_session_email, send_welcome_email};
use crate::mailer::{send_new_confirmation_email, send_password_changed_email};

pub async fn activity(job: ActivityJob) -> Result<(), BoxDynError> {
    let user = commands::get_user_by_id(job.user_id).await?;
    let board = commands::get_board_by_id(job.board_id).await?;

    commands::insert_activity(&user, &board, job.action, job.target_id, &job.data.unwrap_or_default()).await?;

    Ok(())
}

pub async fn new_confirmation(job: NewConfirmationJob) -> Result<(), BoxDynError> {
    let confirmation = commands::get_confirmation_by_id(job.confirmation_id).await?;

    send_new_confirmation_email(&confirmation, &job.code).await?;

    Ok(())
}

pub async fn new_session(job: NewSessionJob) -> Result<(), BoxDynError> {
    let mut session = commands::get_session_by_id(job.session_id).await?;
    let ip_geo = IpGeo::new();

    if let Ok(ip_addr) = IpAddr::from_str(&session.ip_address)
        && !ip_addr.is_loopback()
        && !ip_addr.is_multicast()
        && !ip_addr.is_unspecified()
    {
        let result = ip_geo.info(ip_addr).await;

        if let Ok(ip_geo_info) = result {
            let result = commands::update_session_location(
                &session,
                ip_geo_info.location.country_code2,
                &ip_geo_info.location.state_prov,
                &ip_geo_info.location.city,
            )
            .await;

            if let Ok(updated_session) = result {
                session = updated_session
            }
        }
    };

    send_new_session_email(&session).await?;

    Ok(())
}

pub async fn new_user(job: NewUserJob) -> Result<(), BoxDynError> {
    let user = commands::get_user_by_id(job.user_id).await?;

    let _ = admin_emails::send_new_user_email(&user).await;

    send_welcome_email(&user).await?;

    Ok(())
}

pub async fn password_changed(job: PasswordChangedJob) -> Result<(), BoxDynError> {
    let user = commands::get_user_by_id(job.user_id).await?;

    send_password_changed_email(&user, job.new_password).await?;

    Ok(())
}
