use crate::types::{DownloadId, TopicId, TorrentId};
use serde::{Deserialize, Serialize};

const TASKS_KEY: &str = "tasks";

pub(crate) struct Storage {
    db: sled::Db,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StorageError {
    #[error("Storage error: {0}")]
    DbError(#[from] sled::Error),
}

type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Task {
    pub(crate) topic_id: TopicId,
    pub(crate) download_id: DownloadId,
    pub(crate) torrent_id: TorrentId,
}

pub(crate) trait Tasks {
    fn has_topic(&self, topic_id: &TopicId) -> bool;
}

impl Tasks for Vec<Task> {
    fn has_topic(&self, topic_id: &TopicId) -> bool {
        self.iter().any(|t| &t.topic_id == topic_id)
    }
}

impl Storage {
    pub(crate) fn create(path: &str) -> StorageResult<Self> {
        let db = sled::open(path)?;

        Ok(Self { db })
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) fn get_tasks(&self) -> StorageResult<Vec<Task>> {
        let raw = self.db.get(TASKS_KEY)?;
        let tasks = match raw {
            Some(raw) => serde_json::from_slice(raw.as_ref()).unwrap_or_default(),
            None => vec![],
        };
        Ok(tasks)
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) fn delete_task_by_topic_id(&self, topic_id: &TopicId) -> StorageResult<()> {
        let tasks = self.get_tasks()?;
        self.save_tasks(
            &tasks
                .into_iter()
                .filter(|t| &t.topic_id != topic_id)
                .collect(),
        )
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) fn create_task(&self, task: Task) -> StorageResult<()> {
        let mut tasks = self.get_tasks()?;
        tasks.push(task);
        self.save_tasks(&tasks)
    }

    #[tracing::instrument(err, skip(self))]
    fn save_tasks(&self, tasks: &Vec<Task>) -> StorageResult<()> {
        let vec = serde_json::to_vec(tasks).unwrap();
        let _ = self.db.insert(TASKS_KEY, vec)?;
        Ok(())
    }
}
