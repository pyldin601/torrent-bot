use actix_rt::signal::unix;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use futures_lite::FutureExt;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use torrent_bot_clients::telegram::TelegramBotClient;
use torrent_bot_clients::toloka::TolokaClient;

use crate::config::Config;
use crate::telegram_bot::TelegramBot;

mod config;
mod handlers;
mod serde_helpers;
mod telegram_bot;

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

    let toloka_client = TolokaClient::create(&config.toloka.username, &config.toloka.password)
        .await
        .expect("Unable to initialize toloka client");

    let telegram_client =
        TelegramBotClient::create(config.telegram.bot_token, config.telegram.bot_chat_id);
    let telegram_bot = TelegramBot::create(telegram_client.clone(), toloka_client.clone());

    let server = HttpServer::new({
        let telegram_client = telegram_client.clone();

        move || {
            App::new()
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

    telegram_client.start_repl(telegram_bot);

    actix_rt::spawn({
        async move {

            // while let Some(msg) = rx.next().await {
            //     match msg {
            //         BotCommand::Help => {
            //             telegram_client.send_descriptions().await;
            //         }
            //         BotCommand::Search { query } => {
            //         }
            //         BotCommand::Add { .. } => {
            //             telegram_client.send_message("Soon...").await;
            //         }
            //     }
            // }
        }
    });

    info!("Application started");

    interrupt.recv().or(terminate.recv()).await;

    info!("Received shutdown signal. Shutting down gracefully...");

    // TODO Forceful shutdown

    server_handle.stop(true).await;

    Ok(())
}
