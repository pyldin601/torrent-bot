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

    // Delete removed tasks
    for task in tasks.iter() {
        info!("Checking task {:?}", task);

        if !topics.has_topic(&task.topic_id) {
            info!("Deleting task {:?}", task);

            info!(
                torrent_id = ?task.torrent_id,
                "Removing torrent from torrent client"
            );
            transmission_client
                .remove_with_data(&task.torrent_id)
                .await?;

            info!("Removing task from storage {:?}", task);
            storage.delete_task_by_topic_id(&task.topic_id)?;
        }
    }

    // Check tracked topics
    for topic in topics.iter() {
        info!("Synchronizing topic {:?}", topic);

        let download_id = match toloka_client.get_download_id(&topic.topic_id).await? {
            Some(download_id) => download_id,
            None => {
                info!(?topic, "No download_id in topic; Skipping");
                continue;
            }
        };

        let task = tasks.iter().find(|t| t.topic_id == topic.topic_id);

        match task {
            Some(task) if task.download_id == download_id => {
                info!(?topic, "Found task; task is actual",);
                continue;
            }
            Some(task) => {
                info!(
                    old_download_id = ?task.download_id,
                    new_download_id = ?download_id,
                    "Found task, but download_id has been changed; Will update"
                );
                transmission_client.remove(&task.torrent_id).await?;
            }
            None => {
                info!(?topic, ?download_id, "Creating new task for topic");
            }
        };

        info!(?download_id, "Downloading torrent file");
        let torrent_file_content = toloka_client.download(&download_id).await?;

        info!("Adding torrent file to torrent client");
        match transmission_client
            .add(torrent_file_content, &topic.category)
            .await
        {
            Ok(torrent_id) => {
                let task = Task {
                    topic_id: topic.topic_id.clone(),
                    download_id,
                    torrent_id,
                };

                info!(?task, "Adding task to storage");
                storage.create_task(task)?;
            }
            Err(TransmissionClientError::AlreadyExists) => {
                info!("Torrent file already exists; ignoring")
            }
            Err(error) => {
                info!(?error, "Unable to add torrent file to torrent client");
                return Err(error.into());
            }
        }
    }

    Ok(())
}
