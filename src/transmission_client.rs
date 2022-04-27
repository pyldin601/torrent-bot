use crate::types::{Category, TorrentId};
use transmission_rpc::types::{BasicAuth, Id, RpcResponse, TorrentAddArgs};
use transmission_rpc::TransClient;

pub(crate) struct TransmissionClient {
    client: TransClient,
    download_dir: String,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum TransmissionClientError {
    #[error("Torrent already exists")]
    AlreadyExists,
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
    ) -> Self {
        let client = match (username, password) {
            (Some(user), Some(password)) => {
                TransClient::with_auth(&url, BasicAuth { user, password })
            }
            _ => TransClient::new(&url),
        };

        Self {
            client,
            download_dir,
        }
    }

    pub(crate) async fn add(
        &self,
        torrent_file_content: Vec<u8>,
        category: &Category,
    ) -> TransmissionClientResult<TorrentId> {
        let metainfo = base64::encode(torrent_file_content);

        let RpcResponse { arguments, result } = self
            .client
            .torrent_add(TorrentAddArgs {
                metainfo: Some(metainfo.clone()),
                download_dir: Some(format!(
                    "{}/{}/",
                    self.download_dir.clone(),
                    category.to_string()
                )),
                ..TorrentAddArgs::default()
            })
            .await?;

        if result != "success" {
            return Err(TransmissionClientError::ErroneousResult(result));
        }

        let torrent_added = match arguments.torrent_added {
            Some(torrent_added) => torrent_added,
            None => {
                return Err(TransmissionClientError::AlreadyExists);
            }
        };

        Ok(TorrentId(torrent_added.id.unwrap()))
    }

    pub(crate) async fn remove(&self, torrent_id: &TorrentId) -> TransmissionClientResult<()> {
        let RpcResponse { .. } = self
            .client
            .torrent_remove(vec![Id::Id(**torrent_id)], true)
            .await?;

        Ok(())
    }
}
