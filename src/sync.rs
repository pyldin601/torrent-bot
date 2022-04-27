use crate::storage::{Storage, StorageError};
use crate::toloka_client::{TolokaClient, TolokaClientError, TolokaTopic};
use std::collections::HashSet;
use thiserror::Error;
use transmission_rpc::TransClient;
use transmission_rpc::types::TorrentAddArgs;

enum TrackerTopic {
    Toloka(TolokaTopic),
}

enum Category {
    Movies,
    Series,
    Other(String),
}

enum SyncTask {
    Add(TrackerTopic),
    Remove,
}

#[derive(Debug, Error)]
pub(crate) enum SyncError {
    #[error("Error happened in toloka client: {0}")]
    TolokaClientError(#[from] TolokaClientError),
    #[error("Error happened in storage: {0}")]
    StorageError(#[from] StorageError),
}

impl Into<TrackerTopic> for TolokaTopic {
    fn into(self) -> TrackerTopic {
        TrackerTopic::Toloka(self)
    }
}

impl TrackerTopic {
    fn get_category(&self) -> Category {
        match self {
            TrackerTopic::Toloka(topic) => match topic.category.as_str() {
                "Фільми" => Category::Movies,
                "Серіали" => Category::Series,
                category => Category::Other(category.to_string()),
            },
        }
    }
}

pub(crate) async fn sync(toloka: TolokaClient, storage: Storage, trans_client: TransClient) -> Result<(), SyncError> {
    let topics = toloka.get_watched_topics().await?;
    let mut tasks = storage.get_tasks()?;

    for topic in topics.iter() {
        if tasks.contains_key(&topic.id) {
            continue;
        }

        // Create torrent
        trans_client.torrent_add(TorrentAddArgs {
            metainfo: Some(String::new()),
            ..TorrentAddArgs::default()
        });

        tasks.insert(topic.id.clone(), String::new());
    }

    for (topic_id, torrent_id) in tasks.clone() {
        if topics.iter().all(|t| t.id != topic_id) {
            // Remove torrent

            tasks.remove(&topic_id);
        }
    }

    storage.save_tasks(&tasks)?;

    Ok(())
}
