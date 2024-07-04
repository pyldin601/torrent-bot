use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose};
use parking_lot::Mutex;
use transmission_rpc::TransClient;
use transmission_rpc::types::{
    BasicAuth, Id, RpcResponse, TorrentAddArgs, TorrentAddedOrDuplicate,
};

pub(crate) struct TransmissionClient {
    client: Arc<Mutex<TransClient>>,
    download_dir: String,
    dry_run: bool,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum TransmissionClientError {
    #[error("Torrent already exists")]
    Duplicate,
    #[error("Erroneous result: {0}")]
    ErroneousResult(String),
    #[error("Unable to perform RPC request on transmission server: {0}")]
    TransmissionError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub(crate) type TransmissionClientResult<T> = Result<T, TransmissionClientError>;

impl TransmissionClient {
    pub(crate) fn create(
        url: String,
        username: Option<String>,
        password: Option<String>,
        download_dir: String,
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

    pub(crate) async fn add(
        &self,
        torrent_file_content: Vec<u8>,
        path: &str,
    ) -> TransmissionClientResult<i64> {
        let metainfo = general_purpose::STANDARD.encode(torrent_file_content);
        let dry_run = self.dry_run;

        let RpcResponse {
            arguments,
            result: _,
        } = self
            .client
            .lock()
            .torrent_add(TorrentAddArgs {
                metainfo: Some(metainfo.clone()),
                download_dir: Some(format!("{}/{}/", &self.download_dir, path)),
                paused: Some(dry_run),
                ..TorrentAddArgs::default()
            })
            .await?;

        match arguments {
            TorrentAddedOrDuplicate::TorrentDuplicate(_) => Err(TransmissionClientError::Duplicate),
            TorrentAddedOrDuplicate::TorrentAdded(torrent) => Ok(torrent.id.unwrap()),
        }
    }

    pub(crate) async fn remove_without_data(
        &self,
        torrent_id: i64,
    ) -> TransmissionClientResult<()> {
        let RpcResponse { .. } = self
            .client
            .lock()
            .torrent_remove(vec![Id::Id(torrent_id)], false)
            .await?;

        Ok(())
    }

    pub(crate) async fn remove_with_data(&self, torrent_id: i64) -> TransmissionClientResult<()> {
        let RpcResponse { .. } = self
            .client
            .lock()
            .torrent_remove(vec![Id::Id(torrent_id)], true)
            .await?;

        Ok(())
    }
}
