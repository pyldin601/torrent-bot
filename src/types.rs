use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub(crate) struct TorrentId(pub(crate) i64);

impl Deref for TorrentId {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub(crate) struct TopicId(pub(crate) String);

impl Deref for TopicId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub(crate) struct DownloadId(pub(crate) String, pub(crate) Option<String>);

impl Deref for DownloadId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for DownloadId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}",
            self.0,
            self.1.clone().unwrap_or(format!("none"))
        )
    }
}

#[derive(Debug)]
pub(crate) enum Category {
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

#[derive(Debug)]
pub(crate) struct Topic {
    pub(crate) topic_id: TopicId,
    pub(crate) title: String,
    pub(crate) category: Category,
}

impl Display for Topic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.topic_id.0, self.title)
    }
}

pub(crate) trait Topics {
    fn has_topic(&self, topic_id: &TopicId) -> bool;
}

impl Topics for Vec<Topic> {
    fn has_topic(&self, topic_id: &TopicId) -> bool {
        self.iter().any(|t| &t.topic_id == topic_id)
    }
}
