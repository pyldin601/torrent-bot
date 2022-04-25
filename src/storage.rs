pub(crate) struct Storage {
    db: sled::Db,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StorageError {
    #[error("Database error: {0}")]
    DbError(#[from] sled::Error),
}

type StorageResult<T> = Result<T, StorageError>;

impl Storage {
    pub fn create(path: &str) -> StorageResult<Self> {
        let db = sled::open(path)?;

        Ok(Self { db })
    }
}
