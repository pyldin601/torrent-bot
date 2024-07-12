#[derive(Debug, PartialEq)]
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
