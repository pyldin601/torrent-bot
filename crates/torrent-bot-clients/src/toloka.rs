use reqwest::{Client, StatusCode};
use reqwest::redirect::Policy;
use scraper::{Html, Selector};
use serde::Serialize;
use tracing::warn;

#[derive(Debug)]
pub enum Category {
    Movies,
    Series,
    Other(String),
}

impl ToString for Category {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::Movies => "Movies",
            Self::Series => "Series",
            Self::Other(_) => "Other",
        })
    }
}

pub struct TopicMeta {
    pub topic_id: String,
    pub title: String,
    pub category: Category,
}

pub struct DownloadMeta {
    pub registered_at: String,
    pub download_id: String,
}

pub struct Topic {
    pub topic_meta: TopicMeta,
    pub download_meta: DownloadMeta,
}

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
        let html = Html::parse_document(&document);

        let table_row_selector = Selector::parse(r#"table.forumline tr"#).unwrap();
        let table_entries = html.select(&table_row_selector);

        let href_selector = &Selector::parse(r#"a[href]"#).unwrap();
        let td_selector = &Selector::parse(r#"td"#).unwrap();

        let mut topics = vec![];

        for el in table_entries
            .skip(1)
            .filter(|el| el.children().filter(|el| el.value().is_element()).count() == 6)
            .into_iter()
        {
            let columns = el.select(&td_selector).collect::<Vec<_>>();
            let link = columns[0].select(&href_selector).next().unwrap();

            let category_raw = columns[1]
                .select(&href_selector)
                .next()
                .unwrap()
                .inner_html()
                .to_string();
            let topic_id = link.value().attr("href").unwrap_or_default().to_string();
            let title = link.inner_html().to_string();

            let category = match category_raw.to_lowercase().as_str() {
                s if s.contains("фільм") => Category::Movies,
                s if s.contains("серіал") => Category::Series,
                other => Category::Other(other.to_string()),
            };

            topics.push(TopicMeta {
                topic_id,
                category,
                title,
            });
        }

        Ok(topics)
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
        let html = Html::parse_document(&document);

        let download_selector = Selector::parse(".piwik_download").unwrap();

        let registered_at = {
            let bt_tbl_selector = Selector::parse("table.btTbl").unwrap();
            let bt_row_selector = Selector::parse("tr.row4_to").unwrap();
            let bt_col_selector = Selector::parse("td.genmed").unwrap();

            let parse_registered_at = || -> Option<String> {
                Some(
                    html.select(&bt_tbl_selector)
                        .next()?
                        .select(&bt_row_selector)
                        .skip(1)
                        .next()?
                        .select(&bt_col_selector)
                        .skip(1)
                        .next()?
                        .inner_html()
                        .replace("&nbsp;", ""),
                )
            };

            parse_registered_at()
        };

        Ok(html
            .select(&download_selector)
            .next()
            .map(|e| e.value().attr("href").unwrap_or_default().to_string())
            .map(|url| DownloadMeta {
                download_id: url.replace("download.php?id=", ""),
                registered_at: registered_at.unwrap_or_default(),
            }))
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
