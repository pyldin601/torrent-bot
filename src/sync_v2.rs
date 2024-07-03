use std::collections::HashSet;

use thiserror::Error;
use tracing::{debug, info};

use crate::clients::toloka::{TolokaClient, TolokaClientError};
use crate::clients::transmission::{TransmissionClient, TransmissionClientError};
use crate::task_db::{StorageError, Task, TaskDb};

#[derive(Debug, Error)]
pub(crate) enum SyncError {
    #[error("Error happened in toloka client: {0}")]
    TolokaClientError(#[from] TolokaClientError),
    #[error("Error happened in storage: {0}")]
    StorageError(#[from] StorageError),
    #[error("Error happened in transmission client: {0}")]
    TransmissionError(#[from] TransmissionClientError),
}

pub(crate) async fn sync(
    toloka_client: TolokaClient,
    transmission_client: TransmissionClient,
    task_db: TaskDb,
) -> Result<(), SyncError> {
    debug!("Loading tasks...");
    let tasks = task_db.get_tasks()?;

    debug!("Loading watched topics...");
    let watched_topics = toloka_client.get_watched_topics().await?;

    let mut present_topics = HashSet::new();

    debug!("Syncing present topics...");
    for topic in watched_topics.into_iter() {
        present_topics.insert(topic.topic_meta.topic_id.clone());

        let matched_task = tasks
            .iter()
            .find(|t| t.topic_id == topic.topic_meta.topic_id);

        match matched_task {
            Some(task)
                if task.topic_download_registered_at == topic.download_meta.registered_at =>
            {
                debug!("Topic unchanged: {}", topic.topic_meta.title);
            }
            Some(task) => {
                let torrent_data = toloka_client
                    .download(&topic.download_meta.download_id)
                    .await?;
                transmission_client
                    .remove_without_data(task.transmission_torrent_id)
                    .await?;
                let torrent_id = transmission_client
                    .add(torrent_data, &topic.topic_meta.category.to_string())
                    .await?;

                task_db.delete_task_by_topic_id(&topic.topic_meta.topic_id)?;
                task_db.add_task(Task {
                    topic_id: topic.topic_meta.topic_id,
                    topic_title: topic.topic_meta.title.clone(),
                    topic_download_registered_at: topic.download_meta.registered_at,
                    transmission_torrent_id: torrent_id,
                })?;

                info!("Topic updated: {}", topic.topic_meta.title);
            }
            None => {
                let torrent_data = toloka_client
                    .download(&topic.download_meta.download_id)
                    .await?;
                let torrent_id = transmission_client
                    .add(torrent_data, &topic.topic_meta.category.to_string())
                    .await?;

                task_db.add_task(Task {
                    topic_id: topic.topic_meta.topic_id,
                    topic_title: topic.topic_meta.title.clone(),
                    topic_download_registered_at: topic.download_meta.registered_at,
                    transmission_torrent_id: torrent_id,
                })?;

                info!("Topic added: {}", topic.topic_meta.title);
            }
        }
    }

    debug!("Syncing deleted topics...");
    for task in tasks
        .iter()
        .filter(|t| !present_topics.contains(&t.topic_id))
        .collect::<Vec<_>>()
    {
        transmission_client
            .remove_with_data(task.transmission_torrent_id)
            .await?;
        task_db.delete_task_by_topic_id(&task.topic_id)?;
        info!("Topic deleted: {}", task.topic_title);
    }

    debug!("Done");

    Ok(())
}
