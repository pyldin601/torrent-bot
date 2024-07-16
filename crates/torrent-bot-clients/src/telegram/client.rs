use std::sync::Arc;

use teloxide::{Bot, macros, RequestError};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::utils::command::BotCommands;
use tracing::{error, warn};

use crate::telegram::BotCommandHandler;

#[derive(Clone)]
pub struct TelegramBotClient {
    bot: Bot,
    chat_id: ChatId,
}

#[derive(macros::BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum BotCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "search for a topic.")]
    Search { query: String },
}

pub struct ActionButton {
    pub text: String,
    pub action: String,
}

impl TelegramBotClient {
    pub fn create(bot_token: String, bot_chat_id: i64) -> TelegramBotClient {
        let bot = Bot::new(bot_token);
        let chat_id = ChatId(bot_chat_id);

        TelegramBotClient { bot, chat_id }
    }

    pub fn start_repl(&self, handler: impl BotCommandHandler<'static>) {
        let bot = Clone::clone(&self.bot);
        let handler = Arc::from(handler);

        actix_rt::spawn({
            async move {
                let update_handler = dptree::entry()
                    .branch(Update::filter_callback_query().endpoint({
                        let handler = handler.clone();

                        move |q: CallbackQuery| {
                            let handler = handler.clone();
                            let chat_id = q.chat_id();

                            async move {
                                if let Some(data) = q.data.as_ref() {
                                    if data.starts_with("add_") {
                                        let topic_id = data[4..].to_string();
                                        handler.handle_add_command(&topic_id).await;
                                        return Ok(());
                                    }
                                }

                                warn!(?chat_id, ?q.data, "Unexpected callback data");

                                Ok::<(), RequestError>(())
                            }
                        }
                    }))
                    .branch(Update::filter_message().branch(
                        dptree::entry().filter_command::<BotCommand>().endpoint({
                            let handler = handler.clone();

                            move |bot: Bot, cmd: BotCommand, msg: Message| {
                                let handler = handler.clone();

                                async move {
                                    match cmd {
                                        BotCommand::Help => {
                                            bot.send_message(
                                                msg.chat.id,
                                                BotCommand::descriptions().to_string(),
                                            )
                                            .await?;
                                        }
                                        BotCommand::Search { query } => {
                                            handler.handle_search_command(&query).await;
                                        }
                                    }

                                    Ok::<(), RequestError>(())
                                }
                            }
                        }),
                    ));

                Dispatcher::builder(bot, update_handler)
                    .build()
                    .dispatch()
                    .await
            }
        });
    }

    pub async fn send_message(&self, text: &str) {
        if let Err(error) = self.bot.send_message(self.chat_id, text).await {
            error!(?error, "Failed to send message to telegram bot");
        }
    }

    pub async fn send_message_with_action_buttons(&self, text: &str, buttons: Vec<ActionButton>) {
        let markup = InlineKeyboardMarkup::new(
            buttons
                .into_iter()
                .map(|b| vec![InlineKeyboardButton::callback(b.text, b.action)])
                .collect::<Vec<_>>(),
        );

        if let Err(error) = self
            .bot
            .send_message(self.chat_id, text)
            .reply_markup(markup)
            .await
        {
            error!(?error, "Failed to send message to telegram bot");
        }
    }
}
