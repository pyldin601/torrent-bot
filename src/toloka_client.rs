use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use serde::Serialize;

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

pub(crate) struct TolokaClient {
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TolokaClientError {
    #[error("Login or password is invalid")]
    Unauthorized,
    #[error("Unexpected status code: {0}")]
    Status(StatusCode),
    #[error("Unable to perform http request: {0}")]
    Request(#[from] reqwest::Error),
}

pub(crate) type TolokaClientResult<T> = Result<T, TolokaClientError>;

#[derive(Debug)]
pub(crate) struct Topic {
    pub(crate) id: String,
    pub(crate) category: String,
    pub(crate) title: String,
}

impl TolokaClient {
    pub(crate) async fn create(username: &str, password: &str) -> TolokaClientResult<TolokaClient> {
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

    pub(crate) async fn get_watched_topics(&self) -> TolokaClientResult<Vec<Topic>> {
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
        let selector = Selector::parse(r#"table.forumline tr td"#).unwrap();
        let rows = html.select(&selector).into_iter().collect::<Vec<_>>();

        Ok(rows
            .chunks(6)
            .into_iter()
            .filter(|c| c.len() == 6)
            .map(|c| {
                let link = c[0]
                    .select(&Selector::parse("a[href]").unwrap())
                    .next()
                    .unwrap();
                let category = c[1]
                    .select(&Selector::parse("a[href]").unwrap())
                    .next()
                    .unwrap();

                Topic {
                    id: link.value().attr("href").unwrap().to_string(),
                    category: category.inner_html().to_string(),
                    title: link.inner_html().to_string(),
                }
            })
            .collect())
    }
}
