use crate::config::Config;
use crate::toloka_client::TolokaClient;
use dotenv_codegen::dotenv;

mod config;
mod toloka_client;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let config = Config::from_env();
    let client = TolokaClient::create(&config.toloka.username, &config.toloka.password)
        .await
        .expect("Unable to create TolokaClient");

    let topics = client.get_watched_topics().await.unwrap();

    eprintln!("Res: {:?}", topics);

    Ok(())
}
