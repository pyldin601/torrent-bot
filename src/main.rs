use crate::config::Config;
use crate::storage::Storage;
use crate::sync::sync;
use crate::toloka_client::TolokaClient;
use crate::transmission_client::TransmissionClient;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod storage;
mod sync;
mod toloka_client;
mod transmission_client;
mod types;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = {
        Config::init_dotenv();
        Config::from_env()
    };

    let storage = Storage::create(&config.storage_file).expect("Unable to initialize storage");
    let toloka_client = TolokaClient::create(&config.toloka.username, &config.toloka.password)
        .await
        .expect("Unable to initialize toloka client");
    let transmission_client = TransmissionClient::create(
        config.transmission.url.clone(),
        config.transmission.username.clone(),
        config.transmission.password.clone(),
        config.transmission.download_directory.clone(),
    );

    if let Err(error) = sync(toloka_client, storage, transmission_client).await {
        error!("Error happened during synchronization: {:?}", error);
    }

    Ok(())
}
