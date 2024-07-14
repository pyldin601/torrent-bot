use teloxide::Bot;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::repls::CommandReplExt;
use teloxide::types::Recipient;
use teloxide::utils::command::BotCommands;
use tracing::error;

#[derive(Clone)]
pub struct TelegramBotClient {
    bot: Bot,
    recipient: Recipient,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "search for a topic.")]
    Search { query: String },
    #[command(description = "add topic to bookmarks.")]
    Add { topic_id: String },
}

impl TelegramBotClient {
    pub fn create(bot_token: String, bot_chat_id: i64) -> TelegramBotClient {
        let bot = Bot::new(bot_token);
        let recipient = Recipient::Id(ChatId(bot_chat_id));

        TelegramBotClient { bot, recipient }
    }

    pub async fn start_repl(&self) {
        Command::repl(
            self.bot.clone(),
            |bot: Bot, msg: Message, command: Command| async move {
                match command {
                    Command::Help => {
                        bot.send_message(msg.chat.id, Command::descriptions().to_string())
                            .await?;
                    }
                    Command::Search { query } => {
                        // TODO Search topics
                        bot.send_dice(msg.chat.id).await?;
                    }
                    Command::Add { topic_id } => {
                        // TODO Add topic to bookmarks
                        bot.send_dice(msg.chat.id).await?;
                    }
                }

                return Ok(());
            },
        )
        .await;
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
