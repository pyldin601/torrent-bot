use serde::{Deserialize, Serialize};

const SNIPES_KEY: &str = "torrent_bot_snipes";

pub(crate) struct SnipeDb {
    db: sled::Db,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StorageError {
    #[error("Storage error: {0}")]
    SnipeDbError(#[from] sled::Error),
}

type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Snipe {
    pub(crate) description: String,
}

impl SnipeDb {
    pub(crate) fn create(path: &str) -> StorageResult<Self> {
        let db = sled::open(path)?;

        Ok(Self { db })
    }

    #[tracing::instrument(err, skip(self))]
    pub(crate) fn get_snipes(&self) -> StorageResult<Vec<Snipe>> {
        let raw = self.db.get(SNIPES_KEY)?;
        let tasks = match raw {
            Some(raw) => serde_json::from_slice(raw.as_ref()).unwrap_or_default(),
            None => vec![],
        };

        Ok(tasks)
    }

    #[tracing::instrument(err, skip(self))]
    fn save_snipes(&self, tasks: &Vec<Snipe>) -> StorageResult<()> {
        let vec = serde_json::to_vec(tasks).unwrap();
        let _ = self.db.insert(SNIPES_KEY, vec)?;
        Ok(())
    }
}
