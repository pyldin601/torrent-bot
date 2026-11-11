use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode};
use serde::Serialize;
use serde_json::json;
use tracing::{instrument, warn, Span};

use crate::toloka::types::{DownloadMeta, Topic, TopicMeta};

const TOLOKA_HOST: &str = "https://toloka.to";

#[derive(Serialize)]
struct LoginForm {
    username: String,
    password: String,
    autologin: String,
    ssl: String,
    redirect: String,
    login: String,
}

#[derive(Clone)]
pub struct TolokaClient {
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub enum TolokaClientError {
    #[error("Invalid login or password")]
    Unauthorized,
    #[error("Unexpected status code: {0}")]
    Status(StatusCode),
    #[error("Unable to perform http request: {0}")]
    Request(#[from] reqwest::Error),
}

pub type TolokaClientResult<T> = Result<T, TolokaClientError>;

impl TolokaClient {
    // skip_all: the arguments include the Toloka password.
    #[instrument(name = "toloka.login", skip_all, err, fields(otel.kind = "client"))]
    pub async fn create(username: &str, password: &str) -> TolokaClientResult<TolokaClient> {
        let client = Client::builder()
            .redirect(Policy::none())
            .cookie_store(true)
            // toloka.to (behind Cloudflare) drops keep-alive connections after ~5s idle.
            // We pause 5s between topic requests, so reusing a pooled connection races the
            // server-side close and fails with "Connection reset by peer". Never reuse a
            // connection that has been idle that long.
            .pool_idle_timeout(std::time::Duration::from_secs(2))
            .build()
            .expect("Failed to create HTTP Client");

        let form = LoginForm {
            username: username.to_string(),
            password: password.to_string(),
            autologin: String::from("on"),
            ssl: String::from("on"),
            redirect: String::from("index.php?"),
            login: String::from("Вхід"),
        };

        let response = client
            .post(format!("{}/login.php", TOLOKA_HOST))
            .form(&form)
            .send()
            .await?;

        if response.status() != StatusCode::FOUND {
            return Err(TolokaClientError::Unauthorized);
        }

        Ok(Self { client })
    }

    #[instrument(name = "toloka.download", skip(self), err, fields(otel.kind = "client"))]
    pub async fn download(&self, download_id: &str) -> TolokaClientResult<Vec<u8>> {
        let response = self
            .client
            .get(format!("{}/download.php?id={}", TOLOKA_HOST, download_id))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Err(TolokaClientError::Status(response.status()));
        }

        Ok(response.bytes().await?.to_vec())
    }

    #[instrument(name = "toloka.get_watched_topics_meta", skip_all, err, fields(otel.kind = "client"))]
    async fn get_watched_topics_meta(&self) -> TolokaClientResult<Vec<TopicMeta>> {
        let response = self
            .client
            .get(format!("{}/watched_topics.php", TOLOKA_HOST))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Err(TolokaClientError::Status(response.status()));
        }

        let document = response.text().await?;
        let topics_meta = super::parsers::parse_watched_topics_meta(&document);

        Ok(topics_meta)
    }

    #[instrument(name = "toloka.get_download_meta", skip(self), err, fields(otel.kind = "client"))]
    async fn get_download_meta(&self, topic_id: &str) -> TolokaClientResult<Option<DownloadMeta>> {
        let response = self
            .client
            .get(format!("{}/{}", TOLOKA_HOST, topic_id))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Err(TolokaClientError::Status(response.status()));
        }

        let document = response.text().await?;
        let download_meta = super::parsers::parse_download_meta(&document);

        Ok(download_meta)
    }

    #[instrument(name = "toloka.get_watched_topics", skip_all, err, fields(topics = tracing::field::Empty))]
    pub async fn get_watched_topics(&self) -> TolokaClientResult<Vec<Topic>> {
        let topics_meta = self.get_watched_topics_meta().await?;
        Span::current().record("topics", topics_meta.len());
        let mut topics = vec![];

        for topic_meta in topics_meta.into_iter() {
            let download_meta = self.get_download_meta(&topic_meta.topic_id).await?;
            if download_meta.is_none() {
                warn!(?topic_meta.topic_id, "Missing download meta");
            }
            topics.push(Topic {
                topic_meta,
                download_meta,
            });

            let _ = actix_rt::time::sleep(std::time::Duration::from_secs(5)).await;
        }

        Ok(topics)
    }

    #[instrument(name = "toloka.search", skip(self), err, fields(otel.kind = "client"))]
    pub async fn get_search_results_meta(&self, query: &str) -> TolokaClientResult<Vec<TopicMeta>> {
        let response = self
            .client
            .get(format!("{}/tracker.php", TOLOKA_HOST))
            .query(&json!({ "nm": query }))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Err(TolokaClientError::Status(response.status()));
        }

        let document = response.text().await?;
        let results_meta = super::parsers::parse_search_results_meta(&document);

        Ok(results_meta)
    }

    #[instrument(name = "toloka.add_bookmark", skip(self), err, fields(otel.kind = "client"))]
    pub async fn add_topic_to_bookmarks(&self, topic_id: &str) -> TolokaClientResult<()> {
        let response = self
            .client
            .get(format!("{}/viewtopic.php", TOLOKA_HOST))
            .query(&json!({ "t": topic_id, "watch": "topic" }))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Err(TolokaClientError::Status(response.status()));
        }

        Ok(())
    }

    #[instrument(name = "toloka.remove_bookmark", skip(self), err, fields(otel.kind = "client"))]
    pub async fn remove_topic_from_bookmarks(&self, topic_id: &str) -> TolokaClientResult<()> {
        let response = self
            .client
            .get(format!("{}/viewtopic.php", TOLOKA_HOST))
            .query(&json!({ "t": topic_id, "unwatch": "topic" }))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Err(TolokaClientError::Status(response.status()));
        }

        Ok(())
    }
}
