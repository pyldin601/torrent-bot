use std::sync::Arc;

use base64::{engine::general_purpose, Engine as _};
use parking_lot::Mutex;
use tracing::{debug, instrument};
use transmission_rpc::types::{
    BasicAuth, Id, RpcResponse, TorrentAddArgs, TorrentAddedOrDuplicate,
};
use transmission_rpc::TransClient;

#[derive(Debug)]
pub enum RemoveStrategy {
    KeepLocalData,
    DeleteLocalData,
}

#[derive(Clone)]
pub struct TransmissionClient {
    client: Arc<Mutex<TransClient>>,
    download_dir: Option<String>,
    dry_run: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum TransmissionClientError {
    #[error("Torrent already exists")]
    Duplicate,
    #[error("Unknown error")]
    Error,
    #[error("Erroneous result: {0}")]
    ErroneousResult(String),
    #[error("Missing download dir")]
    MissingDownloadDir,
    #[error("Unable to perform RPC request on transmission server: {0}")]
    TransmissionError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type TransmissionClientResult<T> = Result<T, TransmissionClientError>;

impl TransmissionClient {
    pub fn create(
        url: String,
        username: Option<String>,
        password: Option<String>,
        download_dir: Option<String>,
        dry_run: bool,
    ) -> Self {
        let url = url
            .parse()
            .expect("Unable to parse transmission server url");

        let client = match (username, password) {
            (Some(user), Some(password)) => {
                TransClient::with_auth(url, BasicAuth { user, password })
            }
            _ => TransClient::new(url),
        };

        Self {
            client: Arc::new(Mutex::new(client)),
            download_dir,
            dry_run,
        }
    }

    #[instrument(err, skip(self, torrent_file_content))]
    pub async fn add(
        &self,
        torrent_file_content: Vec<u8>,
        path: &str,
    ) -> TransmissionClientResult<i64> {
        let metainfo = general_purpose::STANDARD.encode(torrent_file_content);
        let dry_run = self.dry_run;
        let download_dir = self
            .download_dir
            .as_ref()
            .ok_or_else(|| TransmissionClientError::MissingDownloadDir)?;

        let RpcResponse {
            arguments,
            result: _,
        } = self
            .client
            .lock()
            .torrent_add(TorrentAddArgs {
                metainfo: Some(metainfo.clone()),
                download_dir: Some(format!("{}/{}/", download_dir, path)),
                paused: Some(dry_run),
                ..TorrentAddArgs::default()
            })
            .await?;

        match arguments {
            TorrentAddedOrDuplicate::TorrentDuplicate(torrent) => Ok(torrent.id.unwrap()),
            TorrentAddedOrDuplicate::TorrentAdded(torrent) => Ok(torrent.id.unwrap()),
            TorrentAddedOrDuplicate::Error => Err(TransmissionClientError::Error),
        }
    }

    #[instrument(err, skip(self))]
    pub async fn remove_without_data(&self, torrent_id: i64) -> TransmissionClientResult<()> {
        let RpcResponse { result, arguments } = self
            .client
            .lock()
            .torrent_remove(vec![Id::Id(torrent_id)], false)
            .await?;

        debug!(
            "Result of torrent_remove call with delete_local_data = false: result = {}, arguments = {:?}",
            result, arguments
        );

        Ok(())
    }

    #[instrument(err, skip(self))]
    pub async fn remove_with_data(&self, torrent_id: i64) -> TransmissionClientResult<()> {
        let RpcResponse { result, arguments } = self
            .client
            .lock()
            .torrent_remove(vec![Id::Id(torrent_id)], true)
            .await?;

        debug!(
            "Result of torrent_remove call with delete_local_data = true: result = {}, arguments = {:?}",
            result, arguments
        );

        Ok(())
    }

    #[instrument(err, skip(self))]
    pub async fn remove(
        &self,
        torrent_id: i64,
        remove_strategy: RemoveStrategy,
    ) -> TransmissionClientResult<()> {
        let RpcResponse {
            result,
            arguments: _,
        } = self
            .client
            .lock()
            .torrent_remove(
                vec![Id::Id(torrent_id)],
                match remove_strategy {
                    RemoveStrategy::KeepLocalData => false,
                    RemoveStrategy::DeleteLocalData => true,
                },
            )
            .await?;

        debug!(?result, "Result of torrent_remove call");

        Ok(())
    }

    #[instrument(err, skip(self))]
    pub async fn get_is_downloaded(&self, torrent_id: i64) -> TransmissionClientResult<bool> {
        let RpcResponse { arguments, .. } = self
            .client
            .lock()
            .torrent_get(None, Some(vec![Id::Id(torrent_id)]))
            .await?;

        Ok(arguments
            .torrents
            .first()
            .map(|torrent| torrent.is_finished)
            .is_some())
    }
}
