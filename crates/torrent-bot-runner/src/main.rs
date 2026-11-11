use tracing::{instrument, Level};

use torrent_bot_clients::telemetry;
use torrent_bot_clients::toloka::TolokaClient;
use torrent_bot_clients::transmission::TransmissionClient;

use crate::client::Client;
use crate::config::Config;
use crate::sync_v2::{sync, SyncError};
use crate::task_db::TaskDb;

mod client;
mod config;
mod sync_extensions;
mod sync_v2;
mod task_db;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let config = {
        Config::init_dotenv();
        Config::from_env()
    };

    let mut telemetry = telemetry::init("torrent-bot-runner", Level::DEBUG);

    // `run` has ended (and its span closed) here, so the shutdown below exports the whole trace.
    if run(config).await.is_err() {
        // process::exit skips destructors, so flush the spans of this run explicitly.
        telemetry.shutdown();
        // Exit non-zero so the Kubernetes Job is marked as failed instead of Completed.
        std::process::exit(1);
    }

    Ok(())
}

/// One run of the CronJob: the root span of its trace, covering setup, login and sync.
#[instrument(name = "run", skip_all, err(Debug))]
async fn run(config: Config) -> Result<(), SyncError> {
    let storage = TaskDb::create(&config.storage_file).expect("Unable to initialize DB");
    let toloka_client = TolokaClient::create(&config.toloka.username, &config.toloka.password)
        .await
        .expect("Unable to initialize toloka client");
    let transmission_client = TransmissionClient::create(
        config.transmission.url,
        config.transmission.username,
        config.transmission.password,
        Some(config.transmission.download_directory),
        config.transmission.dry_run,
    );
    let client = Client::create(&config.server_endpoint);

    sync(
        toloka_client,
        transmission_client,
        storage,
        client,
        config.wipeout_mode,
    )
    .await
}
