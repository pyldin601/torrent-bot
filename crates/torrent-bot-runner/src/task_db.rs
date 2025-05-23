use serde::{Deserialize, Serialize};

const TASKS_KEY: &str = "torrent_bot_tasks";

pub(crate) struct TaskDb {
    db: sled::Db,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StorageError {
    #[error("Storage error: {0}")]
    TaskDbError(#[from] sled::Error),
}

type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum TaskStatus {
    Added,
    Finished,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum TorrentId {
    Id(i64),
    Hash(String),
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Added
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Task {
    pub(crate) topic_id: String,
    pub(crate) topic_title: String,
    pub(crate) topic_download_registered_at: String,
    pub(crate) transmission_torrent_id: TorrentId,
    #[serde(default)]
    pub(crate) task_status: TaskStatus,
}

impl TaskDb {
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
    pub(crate) fn delete_task_by_topic_id(&self, topic_id: &str) -> StorageResult<()> {
        let tasks = self.get_tasks()?;
        self.save_tasks(
            &tasks
                .into_iter()
                .filter(|t| &t.topic_id != topic_id)
                .collect(),
        )
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) fn mark_task_as_finished_by_topic_id(&self, topic_id: &str) -> StorageResult<()> {
        let mut tasks = self.get_tasks()?;

        tasks
            .iter_mut()
            .filter(|task| &task.topic_id == topic_id)
            .for_each(|task| {
                task.task_status = TaskStatus::Finished;
            });

        self.save_tasks(&tasks)
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) fn add_task(&self, task: Task) -> StorageResult<()> {
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
