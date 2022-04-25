use crate::config::Config;
use crate::storage::Storage;
use crate::toloka_client::TolokaClient;

mod config;
mod storage;
mod toloka_client;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let config = Config::from_env();
    let storage = Storage::create(&config.storage_file).expect("Unable to initialize storage");
    let client = TolokaClient::create(&config.toloka.username, &config.toloka.password)
        .await
        .expect("Unable to initialize toloka client");

    let topics = client.get_watched_topics().await.unwrap();

    eprintln!("Res: {:?}", topics);

    Ok(())
}
