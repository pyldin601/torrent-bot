use reqwest::{Client, StatusCode};
use reqwest::redirect::Policy;
use serde::Serialize;
use tracing::warn;

use crate::toloka::types::{DownloadMeta, Topic, TopicMeta};

pub(crate) mod parsers;
pub mod types;

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
    pub async fn create(username: &str, password: &str) -> TolokaClientResult<TolokaClient> {
        let client = Client::builder()
            .redirect(Policy::none())
            .cookie_store(true)
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
        let topics_meta = parsers::parse_watched_topics_meta(&document);

        Ok(topics_meta)
    }

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
        let download_meta = parsers::parse_download_meta(&document);

        Ok(download_meta)
    }

    pub async fn get_watched_topics(&self) -> TolokaClientResult<Vec<Topic>> {
        let topics_meta = self.get_watched_topics_meta().await?;
        let mut topics = vec![];

        for topic_meta in topics_meta.into_iter() {
            match self.get_download_meta(&topic_meta.topic_id).await? {
                Some(download_meta) => {
                    topics.push(Topic {
                        topic_meta,
                        download_meta,
                    });
                }
                None => {
                    warn!(?topic_meta.topic_id, "Missing download meta. Skipping...")
                }
            }
        }

        Ok(topics)
    }
}
