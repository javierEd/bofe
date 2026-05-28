use std::time::Duration;

use apalis::layers::WorkerBuilderExt;
use apalis::prelude::{Monitor, WorkerBuilder};
use tokio::signal::unix::SignalKind;
use tracing::info;

use bofe_core::{jobs_storage, start_tracing_subscriber};

mod config;
mod handlers;
mod ip_geo;
mod mailer;

#[tokio::main]
async fn main() {
    start_tracing_subscriber();

    info!("Monitor starting");

    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt()).expect("Could not create sigint listener");
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate()).expect("Could not create sigterm listener");

    let jobs_storage = jobs_storage().await;

    let new_session_worker = |index| {
        WorkerBuilder::new(format!("new-session-{index}"))
            .backend(jobs_storage.new_session.clone())
            .enable_tracing()
            .concurrency(1)
            .build(handlers::new_session)
    };

    let new_user_worker = |index| {
        WorkerBuilder::new(format!("new-user-{index}"))
            .backend(jobs_storage.new_user.clone())
            .enable_tracing()
            .concurrency(1)
            .build(handlers::new_user)
    };

    let password_changed_worker = |index| {
        WorkerBuilder::new(format!("password-changed-{index}"))
            .backend(jobs_storage.password_changed.clone())
            .enable_tracing()
            .concurrency(1)
            .build(handlers::password_changed)
    };

    Monitor::new()
        .register(new_session_worker)
        .register(new_user_worker)
        .shutdown_timeout(Duration::from_millis(10000))
        .run_with_signal(async {
            info!("Monitor started");

            tokio::select! {
                _ = sigint.recv() => info!("Received SIGINT."),
                _ = sigterm.recv() => info!("Received SIGTERM."),
            };

            info!("Monitor shutting down");

            Ok(())
        })
        .await
        .expect("Monitor failed");

    info!("Monitor shutdown complete");
}
