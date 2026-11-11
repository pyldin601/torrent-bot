use serde_json::json;
use tracing::{error, instrument};

use torrent_bot_clients::telemetry;

pub(crate) struct Client {
    client: reqwest::Client,
    endpoint: String,
}

impl Client {
    pub(crate) fn create(endpoint: &str) -> Client {
        let client = reqwest::Client::builder()
            .build()
            .expect("Unable to create HTTP client");

        Client {
            client,
            endpoint: String::from(endpoint),
        }
    }

    pub async fn send_topic_added(&self, title: &str) {
        self.send_message("Added", title).await
    }

    pub async fn send_topic_deleted(&self, title: &str) {
        self.send_message("Deleted", title).await
    }

    pub async fn send_topic_updated(&self, title: &str) {
        self.send_message("Updated", title).await
    }

    pub async fn send_torrent_downloaded(&self, title: &str) {
        self.send_message("Downloaded", title).await
    }

    #[instrument(name = "server.send_message", skip(self, title), fields(otel.kind = "client"))]
    async fn send_message(&self, kind: &str, title: &str) {
        let result = self
            .client
            .post(format!(
                "{}/internal/telegram-bot/send-message",
                self.endpoint
            ))
            .headers(telemetry::trace_context_headers())
            .json(&json!({ "text": format!("{}: {}", kind, title) }))
            .send()
            .await
            .and_then(|response| response.error_for_status());

        if let Err(error) = result {
            error!(?error, "Failed to send '{}' message", kind);
        }
    }
}
