use teloxide::Bot;
use teloxide::prelude::{ChatId, Requester};
use teloxide::types::Recipient;
use tracing::error;

#[derive(Clone)]
pub struct TelegramBotClient {
    bot: Bot,
    recipient: Recipient,
}

impl TelegramBotClient {
    pub fn create(bot_token: String, bot_chat_id: i64) -> TelegramBotClient {
        let bot = Bot::new(bot_token);
        let recipient = Recipient::Id(ChatId(bot_chat_id));

        TelegramBotClient { bot, recipient }
    }

    pub async fn send_topic_added(&self, title: &str) {
        if let Err(error) = self
            .bot
            .send_message(self.recipient.clone(), format!("Added: {}", title))
            .await
        {
            error!(?error, "Failed to send message to telegram bot");
        }
    }
    pub async fn send_topic_deleted(&self, title: &str) {
        if let Err(error) = self
            .bot
            .send_message(self.recipient.clone(), format!("Deleted: {}", title))
            .await
        {
            error!(?error, "Failed to send message to telegram bot");
        }
    }

    pub async fn send_topic_updated(&self, title: &str) {
        if let Err(error) = self
            .bot
            .send_message(self.recipient.clone(), format!("Updated: {}", title))
            .await
        {
            error!(?error, "Failed to send message to telegram bot");
        }
    }

    pub async fn send_torrent_downloaded(&self, title: &str) {
        if let Err(error) = self
            .bot
            .send_message(self.recipient.clone(), format!("Downloaded: {}", title))
            .await
        {
            error!(?error, "Failed to send message to telegram bot");
        }
    }

    pub async fn send_message(&self, text: &str) {
        if let Err(error) = self.bot.send_message(self.recipient.clone(), text).await {
            error!(?error, "Failed to send message to telegram bot");
        }
    }
}
