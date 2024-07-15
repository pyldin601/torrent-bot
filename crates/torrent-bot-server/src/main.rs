use actix_rt::signal::unix;
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use futures_lite::{FutureExt, StreamExt};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use torrent_bot_clients::telegram::{BotCommand, TelegramBotClient};
use torrent_bot_clients::toloka::TolokaClient;
use torrent_bot_clients::transmission::TransmissionClient;

use crate::config::Config;

mod config;
mod handlers;
mod serde_helpers;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let config = {
        Config::init_dotenv();
        Config::from_env()
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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
    let toloka_client = TolokaClient::create(&config.toloka.username, &config.toloka.password)
        .await
        .expect("Unable to initialize toloka client");

    let telegram_client =
        TelegramBotClient::create(config.telegram.bot_token, config.telegram.bot_chat_id);

    let server = HttpServer::new({
        let telegram_client = telegram_client.clone();

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

    let server_handle = server.handle();

    actix_rt::spawn({
        async move {
            if let Err(error) = server.await {
                error!(?error, "HTTP server initialization failed");
            }
        }
    });

    actix_rt::spawn({
        let mut telegram_client = telegram_client.clone();
        let mut rx = telegram_client.get_command_stream();

        telegram_client.start_repl();

        async move {
            while let Some(msg) = rx.next().await {
                match msg {
                    BotCommand::Help => {
                        telegram_client.send_descriptions().await;
                    }
                    BotCommand::Search { query } => {
                        match toloka_client.get_search_results_meta(&query).await {
                            Ok(results) => {
                                let titles = results
                                    .into_iter()
                                    .map(|t| format!("🔹 {}", t.title))
                                    .collect::<Vec<_>>()
                                    .join("\n");

                                if titles.is_empty() {
                                    return telegram_client.send_message("No results...").await;
                                }

                                telegram_client.send_message(&titles).await;
                            }
                            Err(error) => {
                                error!(?error, "Unable to search topics");
                                telegram_client.send_message("Something wrong").await;
                            }
                        }
                    }
                    BotCommand::Add { .. } => {
                        telegram_client.send_message("Soon...").await;
                    }
                }
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
