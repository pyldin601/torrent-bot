use serde::{Deserialize, Serialize};

const TASKS_KEY: &str = "torrent_bot_tasks";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum TaskStatus {
    Added,
    Finished,
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
    pub(crate) transmission_torrent_id: i64,
    #[serde(default)]
    pub(crate) task_status: crate::task_db::TaskStatus,
}

pub(crate) struct TaskStorage {
    db: sled::Db,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TaskStorageError {
    #[error(transparent)]
    SledError(#[from] sled::Error),
}

type TaskStorageResult<T> = Result<T, TaskStorageError>;

impl TaskStorage {
    pub(crate) fn create(path: &str) -> TaskStorageResult<Self> {
        let db = sled::open(path)?;

        Ok(Self { db })
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) fn get(&self) -> TaskStorageResult<Vec<Task>> {
        let raw = self.db.get(TASKS_KEY)?;
        let tasks = match raw {
            Some(raw) => serde_json::from_slice(raw.as_ref()).unwrap_or_else(|err| {
                tracing::error!(?err, "Unable to deserialize tasks");
                vec![]
            }),
            None => vec![],
        };

        Ok(tasks)
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) fn set(&self, tasks: &Vec<Task>) -> TaskStorageResult<()> {
        let vec = serde_json::to_vec(tasks).unwrap();
        let _ = self.db.insert(TASKS_KEY, vec)?;
        Ok(())
    }
}
