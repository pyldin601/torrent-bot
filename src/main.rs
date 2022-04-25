use crate::toloka_client::TolokaClient;
use dotenv_codegen::dotenv;

mod toloka_client;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let client = TolokaClient::create(dotenv!("TOLOKA_USERNAME"), dotenv!("TOLOKA_PASSWORD"))
        .await
        .unwrap();
    let topics = client.get_watched_topics().await.unwrap();

    eprintln!("Res: {:?}", topics);

    Ok(())
}
