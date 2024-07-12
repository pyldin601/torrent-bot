use scraper::{Html, Selector};

use crate::toloka::types::{Category, DownloadMeta, TopicMeta};

pub(crate) fn parse_watched_topics_meta(document: &str) -> Vec<TopicMeta> {
    let html = Html::parse_document(document);

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

    topics
}

pub(crate) fn parse_download_meta(document: &str) -> Option<DownloadMeta> {
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

    html.select(&download_selector)
        .next()
        .map(|e| e.value().attr("href").unwrap_or_default().to_string())
        .map(|url| DownloadMeta {
            download_id: url.replace("download.php?id=", ""),
            registered_at: registered_at.unwrap_or_default(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_parse_of_watched_topics_meta() {
        let document = include_str!("./res/watched_topics.html");
        let topics_meta = parse_watched_topics_meta(document);

        assert_eq!(topics_meta.len(), 20);

        assert_eq!(topics_meta[0].topic_id, "t679577");
        assert_eq!(topics_meta[0].title, "Дім Дракона (Сезон 2, серія 1-4) / House of the Dragon (Season 2) (2024) WEB-DL 1080p Ukr/Eng | sub Ukr/Multi");
        assert_eq!(topics_meta[0].category, Category::Series);
    }

    #[actix_rt::test]
    async fn test_parse_of_download_meta() {
        let document = include_str!("./res/single_topic.html");
        let download_meta = parse_download_meta(document).unwrap();

        assert_eq!(download_meta.download_id, "693501");
        assert_eq!(download_meta.registered_at, "2024-07-08 14:53");
    }
}
