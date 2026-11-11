use std::collections::HashSet;

use thiserror::Error;
use tracing::{debug, info, info_span, instrument, warn, Instrument};

use torrent_bot_clients::toloka;
use torrent_bot_clients::transmission;

use crate::client::Client;
use crate::task_db::{StorageError, Task, TaskDb, TaskStatus};

#[derive(Debug, Error)]
pub(crate) enum SyncError {
    #[error("Error happened in toloka client: {0}")]
    TolokaClientError(#[from] toloka::TolokaClientError),
    #[error("Error happened in storage: {0}")]
    StorageError(#[from] StorageError),
    #[error("Error happened in transmission client: {0}")]
    TransmissionError(#[from] transmission::TransmissionClientError),
    #[error("Toloka returned no watched topics while {0} tasks exist; refusing to delete them")]
    EmptyWatchedTopics(usize),
}

#[instrument(name = "sync", skip_all, err, fields(wipeout_mode))]
pub(crate) async fn sync(
    toloka_client: toloka::TolokaClient,
    transmission_client: transmission::TransmissionClient,
    task_db: TaskDb,
    client: Client,
    wipeout_mode: bool,
) -> Result<(), SyncError> {
    debug!("Loading tasks...");
    let tasks = task_db.get_tasks()?;

    debug!("Loading watched topics...");
    let watched_topics = toloka_client.get_watched_topics().await?;
    let watched_topics_ids = watched_topics
        .iter()
        .map(|t| t.topic_meta.topic_id.clone())
        .collect::<HashSet<_>>();

    // An empty list almost always means we got a login or Cloudflare page instead of the real
    // one. Deleting every task (and its local data) based on that would be destructive.
    if watched_topics.is_empty() && !tasks.is_empty() {
        return Err(SyncError::EmptyWatchedTopics(tasks.len()));
    }

    if !wipeout_mode {
        debug!("Syncing present topics...");
        for topic in watched_topics.into_iter() {
            let Some(download_meta) = topic.download_meta else {
                warn!(
                    "Skipping topic without download meta: {}",
                    topic.topic_meta.title
                );
                continue;
            };

            let matched_task = tasks
                .iter()
                .find(|t| t.topic_id == topic.topic_meta.topic_id);

            let span = info_span!(
                "sync_topic",
                topic_id = %topic.topic_meta.topic_id,
                title = %topic.topic_meta.title,
            );

            async {
                match matched_task {
                    Some(task)
                        if task.topic_download_registered_at == download_meta.registered_at =>
                    {
                        debug!("Topic unchanged: {}", topic.topic_meta.title);

                        if matches!(task.task_status, TaskStatus::Added) {
                            let torrent_id = (&task.transmission_torrent_id).into();
                            let is_downloaded =
                                transmission_client.get_is_downloaded(&torrent_id).await?;

                            if is_downloaded {
                                task_db.mark_task_as_finished_by_topic_id(&task.topic_id)?;

                                client.send_torrent_downloaded(&task.topic_title).await;

                                info!("Torrent downloaded: {}", topic.topic_meta.title);
                            }
                        }
                    }
                    Some(task) => {
                        let torrent_data =
                            toloka_client.download(&download_meta.download_id).await?;
                        let torrent_id = (&task.transmission_torrent_id).into();
                        transmission_client
                            .remove(&torrent_id, transmission::RemoveStrategy::KeepLocalData)
                            .await?;
                        let torrent_id = transmission_client
                            .add(torrent_data, &topic.topic_meta.category.to_string())
                            .await?;

                        task_db.delete_task_by_topic_id(&topic.topic_meta.topic_id)?;
                        task_db.add_task(Task {
                            topic_id: topic.topic_meta.topic_id,
                            topic_title: topic.topic_meta.title.clone(),
                            topic_download_registered_at: download_meta.registered_at,
                            transmission_torrent_id: (&torrent_id).into(),
                            task_status: TaskStatus::Added,
                        })?;

                        client.send_topic_updated(&topic.topic_meta.title).await;

                        info!("Topic updated: {}", topic.topic_meta.title);
                    }
                    None => {
                        let torrent_data =
                            toloka_client.download(&download_meta.download_id).await?;
                        let torrent_id = transmission_client
                            .add(torrent_data, &topic.topic_meta.category.to_string())
                            .await?;

                        task_db.add_task(Task {
                            topic_id: topic.topic_meta.topic_id,
                            topic_title: topic.topic_meta.title.clone(),
                            topic_download_registered_at: download_meta.registered_at,
                            transmission_torrent_id: (&torrent_id).into(),
                            task_status: TaskStatus::Added,
                        })?;

                        client.send_topic_added(&topic.topic_meta.title).await;

                        info!("Topic added: {}", topic.topic_meta.title);
                    }
                }

                Ok::<(), SyncError>(())
            }
            .instrument(span)
            .await?;
        }
    }

    debug!("Syncing deleted topics...");
    for task in tasks
        .iter()
        .filter(|t| !watched_topics_ids.contains(&t.topic_id))
    {
        let span = info_span!(
            "delete_topic",
            topic_id = %task.topic_id,
            title = %task.topic_title,
        );

        async {
            let torrent_id = (&task.transmission_torrent_id).into();
            transmission_client
                .remove(&torrent_id, transmission::RemoveStrategy::DeleteLocalData)
                .await?;
            task_db.delete_task_by_topic_id(&task.topic_id)?;

            client.send_topic_deleted(&task.topic_title).await;

            info!("Topic deleted: {}", task.topic_title);

            Ok::<(), SyncError>(())
        }
        .instrument(span)
        .await?;
    }

    debug!("Done");

    Ok(())
}
