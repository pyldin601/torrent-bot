use serde_json::json;
use tracing::error;

pub(crate) struct Client {
    client: reqwest::Client,
    endpoint: String,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ClientError {
    #[error("Unable to perform HTTP request: {0}")]
    Request(#[from] reqwest::Error),
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
        let text = format!("Added: {}", title);

        if let Err(error) = self
            .client
            .post(format!(
                "{}/internal/telegram-bot/send-message",
                self.endpoint
            ))
            .json(&json!({
                "text": text
            }))
            .send()
            .await
        {
            error!(?error, "Failed to send 'Added' message");
        }
    }
    pub async fn send_topic_deleted(&self, title: &str) {
        let text = format!("Deleted: {}", title);

        if let Err(error) = self
            .client
            .post(format!(
                "{}/internal/telegram-bot/send-message",
                self.endpoint
            ))
            .json(&json!({
                "text": text
            }))
            .send()
            .await
        {
            error!(?error, "Failed to send 'Deleted' message");
        }
    }

    pub async fn send_topic_updated(&self, title: &str) {
        let text = format!("Updated: {}", title);

        if let Err(error) = self
            .client
            .post(format!(
                "{}/internal/telegram-bot/send-message",
                self.endpoint
            ))
            .json(&json!({
                "text": text
            }))
            .send()
            .await
        {
            error!(?error, "Failed to send 'Updated' message");
        }
    }

    pub async fn send_torrent_downloaded(&self, title: &str) {
        let text = format!("Downloaded: {}", title);

        if let Err(error) = self
            .client
            .post(format!(
                "{}/internal/telegram-bot/send-message",
                self.endpoint
            ))
            .json(&json!({
                "text": text
            }))
            .send()
            .await
        {
            error!(?error, "Failed to send 'Downloaded' message");
        }
    }
}
