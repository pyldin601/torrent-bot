use crate::config::Config;
use crate::storage::Storage;
use crate::toloka_client::TolokaClient;
use transmission_rpc::types::BasicAuth;
use transmission_rpc::TransClient;

mod config;
mod storage;
mod sync;
mod toloka_client;
mod transmission_client;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let config = Config::from_env();
    let storage = Storage::create(&config.storage_file).expect("Unable to initialize storage");
    let client = TolokaClient::create(&config.toloka.username, &config.toloka.password)
        .await
        .expect("Unable to initialize toloka client");
    let trans = match (config.transmission.username, config.transmission.password) {
        (Some(username), Some(password)) => TransClient::with_auth(
            &config.transmission.url,
            BasicAuth {
                user: username,
                password: password,
            },
        ),
        _ => TransClient::new(&config.transmission.url),
    };

    let stats = trans.session_get().await;

    eprintln!("{:?}", stats);

    Ok(())
}
