use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

use crate::clients::toloka::TolokaClient;
use crate::clients::transmission::TransmissionClient;
use crate::config::Config;
use crate::sync_v2::sync;
use crate::task_db::TaskDb;

mod clients;
mod config;
mod sync_v2;
mod task_db;

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

    let storage = TaskDb::create(&config.storage_file).expect("Unable to initialize DB");
    let toloka_client = TolokaClient::create(&config.toloka.username, &config.toloka.password)
        .await
        .expect("Unable to initialize toloka client");
    let transmission_client = TransmissionClient::create(
        config.transmission.url.clone(),
        config.transmission.username.clone(),
        config.transmission.password.clone(),
        config.transmission.download_directory.clone(),
        config.transmission.dry_run,
    );

    if let Err(error) = sync(
        toloka_client,
        transmission_client,
        storage,
        config.wipeout_mode,
    )
    .await
    {
        error!("Sync error: {:?}", error);
    }

    Ok(())
}
