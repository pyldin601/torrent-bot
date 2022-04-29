use crate::storage::{Storage, StorageError, Task};
use crate::toloka_client::{TolokaClient, TolokaClientError};
use crate::transmission_client::TransmissionClientError;
use crate::types::Topics;
use crate::TransmissionClient;
use thiserror::Error;
use tracing::{debug, info, warn};

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
    storage: Storage,
    transmission_client: TransmissionClient,
) -> Result<(), SyncError> {
    let topics = toloka_client.get_watched_topics().await?;
    let tasks = storage.get_tasks()?;

    // Remove uninterested tasks
    for task in tasks.iter() {
        if topics.has_topic(&task.topic_id) {
            info!(
                "Removing task; topic_id={:?}, torrent_id={:?}",
                task.topic_id, task.torrent_id
            );
            transmission_client
                .remove_with_data(&task.torrent_id)
                .await?;
            storage.delete_task_by_topic_id(&task.topic_id)?;
        }
    }

    for topic in topics.iter() {
        debug!(
            "Synchronizing topic; topic_id={:?}, title={:?}",
            topic.topic_id, topic.title
        );
        let download_id = match toloka_client.get_download_id(&topic.topic_id).await? {
            Some(download_id) => download_id,
            None => {
                debug!(
                    "No download_id for topic_id={:?}; Skipping...",
                    topic.topic_id
                );
                continue;
            }
        };

        let task = tasks.iter().find(|t| t.topic_id == topic.topic_id);

        match task {
            Some(task) if task.download_id == download_id => {
                debug!(
                    "Task already exist; topic={:?}, download_id={:?}",
                    topic, download_id
                );
                continue;
            }
            Some(task) => {
                info!(
                    "Task exist, but download id has been changed; topic={:?}, download_id={:?}",
                    topic, download_id
                );
                transmission_client.remove(&task.torrent_id).await?;
            }
            None => {
                info!(
                    "Creating new task; topic={:?}, download_id={:?}",
                    topic, download_id
                );
            }
        };

        debug!("Downloading torrent file; download_id={:?}", download_id);
        let torrent_file_content = toloka_client.download(&download_id).await?;

        debug!("Adding torrent file...");
        match transmission_client
            .add(torrent_file_content, &topic.category)
            .await
        {
            Ok(torrent_id) => {
                storage.create_task(Task {
                    topic_id: topic.topic_id.clone(),
                    download_id,
                    torrent_id,
                })?;
            }
            Err(TransmissionClientError::AlreadyExists) => {
                warn!("Torrent file already exists; ignoring")
            }
            Err(error) => {
                warn!("Unable to add torrent file; error={:?}", error);
                return Err(error.into());
            }
        }
    }

    Ok(())
}
