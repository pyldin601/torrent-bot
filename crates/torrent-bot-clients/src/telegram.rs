use teloxide::Bot;
use teloxide::prelude::{ChatId, Requester};
use teloxide::types::Recipient;
use tracing::error;

#[derive(Clone)]
pub enum TelegramBotClient {
    Active { bot: Bot, recipient: Recipient },
    Inactive,
}

impl TelegramBotClient {
    pub fn create(bot_token: Option<String>, bot_chat_id: Option<i64>) -> TelegramBotClient {
        match (bot_token, bot_chat_id) {
            (Some(bot_token), Some(chat_id)) => {
                let bot = Bot::new(bot_token);
                let recipient = Recipient::Id(ChatId(chat_id));

                TelegramBotClient::Active { bot, recipient }
            }
            _ => TelegramBotClient::Inactive,
        }
    }

    pub async fn send_topic_added(&self, title: &str) {
        if let TelegramBotClient::Active { bot, recipient } = self {
            if let Err(error) = bot
                .send_message(recipient.clone(), format!("Added: {}", title))
                .await
            {
                error!(?error, "Failed to send message to telegram bot");
            }
        }
    }
    pub async fn send_topic_deleted(&self, title: &str) {
        if let TelegramBotClient::Active { bot, recipient } = self {
            if let Err(error) = bot
                .send_message(recipient.clone(), format!("Deleted: {}", title))
                .await
            {
                error!(?error, "Failed to send message to telegram bot");
            }
        }
    }

    pub async fn send_topic_updated(&self, title: &str) {
        if let TelegramBotClient::Active { bot, recipient } = self {
            if let Err(error) = bot
                .send_message(recipient.clone(), format!("Updated: {}", title))
                .await
            {
                error!(?error, "Failed to send message to telegram bot");
            }
        }
    }

    pub async fn send_torrent_downloaded(&self, title: &str) {
        if let TelegramBotClient::Active { bot, recipient } = self {
            if let Err(error) = bot
                .send_message(recipient.clone(), format!("Downloaded: {}", title))
                .await
            {
                error!(?error, "Failed to send message to telegram bot");
            }
        }
    }

    pub async fn send_message(&self, text: &str) {
        if let TelegramBotClient::Active { bot, recipient } = self {
            if let Err(error) = bot.send_message(recipient.clone(), text).await {
                error!(?error, "Failed to send message to telegram bot");
            }
        }
    }
}
