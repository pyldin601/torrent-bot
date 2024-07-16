use tracing::error;

use torrent_bot_clients::telegram::{BotCommandHandler, TelegramBotClient};
use torrent_bot_clients::toloka::TolokaClient;

pub(crate) struct TelegramBot {
    client: TelegramBotClient,
    toloka: TolokaClient,
}

impl TelegramBot {
    pub(crate) fn create(client: TelegramBotClient, toloka: TolokaClient) -> Self {
        TelegramBot { client, toloka }
    }
}

#[async_trait::async_trait]
impl BotCommandHandler<'_> for TelegramBot {
    async fn handle_search_command(&self, query: &str) {
        match self.toloka.get_search_results_meta(&query).await {
            Ok(results) => {
                let titles = results
                    .into_iter()
                    .map(|t| format!("🔹 {}", t.title))
                    .take(10)
                    .collect::<Vec<_>>()
                    .join("\n");

                if titles.is_empty() {
                    return self.client.send_message("No results...").await;
                }

                self.client.send_message(&titles).await;
            }
            Err(error) => {
                error!(?error, "Unable to search topics");
                self.client
                    .send_message("Something wrong... Check the logs.")
                    .await;
            }
        }
    }
}
