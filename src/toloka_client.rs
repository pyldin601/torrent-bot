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

        let table_row_selector = Selector::parse(r#"table.forumline tr"#).unwrap();
        let table_entries = html.select(&table_row_selector);

        let href_selector = &Selector::parse(r#"a[href]"#).unwrap();
        let td_selector = &Selector::parse(r#"td"#).unwrap();

        Ok(table_entries
            .skip(1)
            .filter(|el| el.children().filter(|el| el.value().is_element()).count() == 6)
            .map(|el| {
                let columns = el.select(&td_selector).collect::<Vec<_>>();
                let link = columns[0].select(&href_selector).next().unwrap();
                let category = columns[1].select(&href_selector).next().unwrap();

                Topic {
                    id: link.value().attr("href").unwrap_or_default().to_string(),
                    category: category.inner_html().to_string(),
                    title: link.inner_html().to_string(),
                }
            })
            .collect())
    }
}
