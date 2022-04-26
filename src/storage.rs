use std::collections::HashMap;

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

pub(crate) type Tasks = HashMap<String, String>;

impl Storage {
    pub fn create(path: &str) -> StorageResult<Self> {
        let db = sled::open(path)?;

        Ok(Self { db })
    }

    pub fn get_tasks(&self) -> StorageResult<Tasks> {
        let raw = self.db.get(TASKS_KEY)?.unwrap_or_default();

        Ok(serde_json::from_slice::<Tasks>(raw.as_ref()).unwrap_or_default())
    }

    pub fn save_tasks(&self, tasks: &Tasks) -> StorageResult<()> {
        let vec = serde_json::to_vec(tasks).unwrap();

        let _ = self.db.insert(TASKS_KEY, vec)?;

        Ok(())
    }
}
