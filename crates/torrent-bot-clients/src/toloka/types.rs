use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Category {
    Movies,
    Series,
    Other(String),
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = String::from(match self {
            Self::Movies => "Movies",
            Self::Series => "Series",
            Self::Other(_) => "Other",
        });
        write!(f, "{}", str)
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
