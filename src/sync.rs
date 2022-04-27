use crate::storage::{Storage, StorageError, Task};
use crate::toloka_client::{TolokaClient, TolokaClientError};
use crate::transmission_client::TransmissionClientError;
use crate::TransmissionClient;
use thiserror::Error;

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
    eprintln!("Reading topics...");
    let topics = toloka_client.get_watched_topics().await?;
    eprintln!("Reading tasks...");
    let tasks = storage.get_tasks()?;

    for topic in topics.iter() {
        if tasks.iter().any(|t| t.topic_id == topic.topic_id) {
            continue;
        }

        eprintln!("Creating new task... {:?}", topic);

        let download_id = match toloka_client.get_download_id(&topic.topic_id).await? {
            Some(download_id) => download_id,
            None => continue,
        };

        let torrent_file_content = toloka_client.download(&download_id).await?;
        let torrent_id = transmission_client
            .add(torrent_file_content, &topic.category)
            .await?;

        storage.create_task(Task {
            topic_id: topic.topic_id.clone(),
            torrent_id,
        })?;
    }

    for Task {
        topic_id,
        torrent_id,
    } in tasks.clone()
    {
        if topics.iter().all(|t| t.topic_id != topic_id) {
            eprintln!("Removing old task... (topic_id={})", *topic_id);

            transmission_client.remove(&torrent_id).await?;
            storage.delete_task_by_topic_id(&topic_id)?;
        }
    }

    Ok(())
}
