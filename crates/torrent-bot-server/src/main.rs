use std::sync::Arc;

use actix_rt::signal::unix;
use actix_web::{App, HttpServer, web};
use actix_web::web::Data;
use futures_lite::FutureExt;
use tracing::{error, info};

use torrent_bot_clients::transmission::TransmissionClient;

use crate::config::Config;

mod config;
mod handlers;

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

    let transmission_client = Arc::new(TransmissionClient::create(
        config.transmission.url,
        config.transmission.username,
        config.transmission.password,
        None,
        false,
    ));

    let server = HttpServer::new({
        move || {
            App::new()
                .app_data(Data::new(Arc::clone(&transmission_client)))
                .service(web::resource("/").route(web::get().to(handlers::dummy::dummy)))
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

    info!("Application started");

    interrupt.recv().or(terminate.recv()).await;

    info!("Received shutdown signal. Shutting down gracefully...");

    // TODO Forceful shutdown

    server_handle.stop(true).await;

    Ok(())
}
