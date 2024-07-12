use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;

use torrent_bot_clients::telegram::TelegramBotClient;

#[derive(Deserialize)]
pub(crate) struct SendMessageJson {
    text: String,
}

pub(crate) async fn send_message(
    json: web::Json<SendMessageJson>,
    telegram_bot: web::Data<TelegramBotClient>,
) -> impl Responder {
    telegram_bot.send_message(&json.text).await;

    HttpResponse::Ok().finish()
}
