use crate::storage::{Storage, StorageError, Task};
use crate::toloka_client::{TolokaClient, TolokaClientError};
use crate::transmission_client::TransmissionClientError;
use crate::types::Topics;
use crate::TransmissionClient;
use thiserror::Error;
use tracing::info;

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
    web_client: TolokaClient,
    storage: Storage,
    transmission_client: TransmissionClient,
) -> Result<(), SyncError> {
    info!("Synchronizing...");

    let watched_topics = web_client.get_watched_topics().await?;
    let tasks = storage.get_tasks()?;

    let tasks_to_delete = tasks
        .iter()
        .filter(|t| !watched_topics.has_topic(&t.topic_id))
        .collect::<Vec<_>>();

    info!("Topics to delete: {:?}", tasks_to_delete);

    for task in &tasks_to_delete {
        transmission_client
            .remove_with_data(&task.torrent_id)
            .await?;
        storage.delete_task_by_topic_id(&task.topic_id)?;
    }

    info!("Untracked topics deleted: {}", tasks_to_delete.len());

    info!("Checking topics...");

    // Check tracked topics
    for topic in watched_topics.iter() {
        let download_id = match web_client.get_download_id(&topic.topic_id).await? {
            Some(download_id) => download_id,
            None => {
                info!(?topic, "No download_id in topic; Skipping");
                continue;
            }
        };

        let task = tasks.iter().find(|t| t.topic_id == topic.topic_id);

        match task {
            Some(task) if task.download_id == download_id => {
                info!("Topic {} hasn't changed", topic);
                continue;
            }
            Some(task) => {
                info!(
                    "Topic {} has changed: {} != {}",
                    topic, task.download_id, download_id
                );
                transmission_client.remove(&task.torrent_id).await?;
            }
            None => {
                info!("New topic {}", topic);
            }
        };

        info!(?download_id, "Downloading torrent file");
        let torrent_file_content = web_client.download(&download_id).await?;

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

    info!("Topics check completed");

    Ok(())
}
