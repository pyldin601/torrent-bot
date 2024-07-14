use std::sync::Arc;

use actix_rt::signal::unix;
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use futures_lite::FutureExt;
use tracing::{error, info};

use torrent_bot_clients::telegram::TelegramBotClient;
use torrent_bot_clients::transmission::TransmissionClient;

use crate::config::Config;

mod config;
mod handlers;
mod serde_helpers;
mod telegram_service;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let config = {
        Config::init_dotenv();
        Config::from_env()
    };

    let mut terminate = unix::signal(unix::SignalKind::terminate())?;
    let mut interrupt = unix::signal(unix::SignalKind::interrupt())?;

    let shutdown_timeout = config.shutdown_timeout.clone();
    let bind_address = config.bind_address.clone();

    let transmission_client = TransmissionClient::create(
        config.transmission.url,
        config.transmission.username,
        config.transmission.password,
        None,
        false,
    );

    let telegram_client =
        TelegramBotClient::create(config.telegram.bot_token, config.telegram.bot_chat_id);

    let server = HttpServer::new({
        move || {
            App::new()
                .app_data(Data::new(Clone::clone(&transmission_client)))
                .app_data(Data::new(Clone::clone(&telegram_client)))
                .service(
                    web::resource("/internal/telegram-bot/send-message")
                        .route(web::post().to(handlers::telegram_bot::send_message)),
                )
                .route(
                    "/health/alive",
                    web::get().to(handlers::readiness_check::readiness_check),
                )
                .route(
                    "/health/ready",
                    web::get().to(handlers::readiness_check::readiness_check),
                )
        }
    })
    .shutdown_timeout(shutdown_timeout)
    .bind(bind_address)?
    .run();

    actix_rt::spawn({
        let telegram_client = telegram_client.clone();

        async move { telegram_client.start_repl().await }
    });

    let server_handle = server.handle();

    actix_rt::spawn({
        async move {
            if let Err(error) = server.await {
                error!(?error, "HTTP server initialization failed");
            }
        }
    });

    info!("Application started");

    interrupt.recv().or(terminate.recv()).await;

    info!("Received shutdown signal. Shutting down gracefully...");

    // TODO Forceful shutdown

    server_handle.stop(true).await;

    Ok(())
}
