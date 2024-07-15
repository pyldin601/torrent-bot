use actix_rt::task::JoinHandle;
use futures::channel::mpsc;
use futures::SinkExt;
use teloxide::{Bot, macros};
use teloxide::prelude::*;
use teloxide::types::Recipient;
use teloxide::utils::command::BotCommands;
use tracing::error;

#[derive(Clone)]
pub struct TelegramBotClient {
    bot: Bot,
    recipient: Recipient,
    command_stream: Option<mpsc::Sender<BotCommand>>,
}

#[derive(macros::BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum BotCommand {
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

        TelegramBotClient {
            bot,
            recipient,
            command_stream: None,
        }
    }

    pub fn get_command_stream(&mut self) -> mpsc::Receiver<BotCommand> {
        let (tx, rx) = mpsc::channel(0);
        self.command_stream.replace(tx);
        rx
    }

    pub fn start_repl(&mut self) -> Option<JoinHandle<()>> {
        if let Some(tx) = self.command_stream.take() {
            return Some(actix_rt::spawn(BotCommand::repl(
                self.bot.clone(),
                move |bot_command: BotCommand| {
                    let mut tx = tx.clone();

                    async move {
                        if let Err(error) = tx.send(bot_command).await {
                            error!(?error, "Failed sending command to the channel");
                        }

                        Ok(())
                    }
                },
            )));
        }

        None
    }

    pub async fn send_message(&self, text: &str) {
        if let Err(error) = self.bot.send_message(self.recipient.clone(), text).await {
            error!(?error, "Failed to send message to telegram bot");
        }
    }

    pub async fn send_descriptions(&self) {
        if let Err(error) = self
            .bot
            .send_message(
                self.recipient.clone(),
                BotCommand::descriptions().to_string(),
            )
            .await
        {
            error!(?error, "Failed to send descriptions to telegram bot");
        }
    }
}
