use tracing::error;

use torrent_bot_clients::telegram::{ActionButton, BotCommandHandler, TelegramBotClient};
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
                if results.is_empty() {
                    self.client.send_message("No results...").await;
                    return;
                }

                self.client
                    .send_message_with_action_buttons(
                        "Found results:",
                        results
                            .into_iter()
                            .take(10)
                            .map(|t| ActionButton {
                                text: t.title,
                                action: format!("add_{}", &t.topic_id[1..]),
                            })
                            .collect::<Vec<_>>(),
                    )
                    .await;
            }
            Err(error) => {
                error!(?error, "Unable to search topics");
                self.client
                    .send_message("Something went wrong... Check the logs.")
                    .await;
            }
        }
    }

    async fn handle_add_command(&self, topic_id: &str) {
        match self.toloka.add_topic_to_bookmarks(topic_id).await {
            Ok(()) => {
                self.client.send_message("Bookmarked 👍").await;
            }
            Err(error) => {
                error!(?error, "Unable to add topic to bookmarks");

                self.client
                    .send_message("Something went wrong... Check the logs.")
                    .await;
            }
        }
    }
}
