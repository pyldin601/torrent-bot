use crate::config::Config;
use crate::storage::Storage;
use crate::sync::sync;
use crate::toloka_client::TolokaClient;
use crate::transmission_client::TransmissionClient;

mod config;
mod storage;
mod sync;
mod toloka_client;
mod transmission_client;
mod types;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let config = Config::from_env();
    let storage = Storage::create(&config.storage_file).expect("Unable to initialize storage");
    let client = TolokaClient::create(&config.toloka.username, &config.toloka.password)
        .await
        .expect("Unable to initialize toloka client");
    let trans = TransmissionClient::create(
        config.transmission.url.clone(),
        config.transmission.username.clone(),
        config.transmission.password.clone(),
        config.transmission.download_directory.clone(),
    );

    if let Err(error) = sync(client, storage, trans).await {
        eprintln!("Error: {:?}", error);
    }

    Ok(())
}
