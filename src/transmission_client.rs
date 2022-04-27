use serde::{Deserialize, Serialize};
use std::ops::Deref;
use transmission_rpc::types::{BasicAuth, Id, RpcResponse, TorrentAddArgs};
use transmission_rpc::TransClient;

pub(crate) struct TransmissionClient {
    client: TransClient,
    download_dir: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TorrentId(i64);

impl Deref for TorrentId {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
        torrent_file_content: String,
    ) -> transmission_rpc::types::Result<TorrentId> {
        let RpcResponse { arguments, .. } = self
            .client
            .torrent_add(TorrentAddArgs {
                metainfo: Some(torrent_file_content),
                download_dir: Some(self.download_dir.clone()),
                ..TorrentAddArgs::default()
            })
            .await?;

        Ok(TorrentId(arguments.torrent_added.unwrap().id.unwrap()))
    }

    pub(crate) async fn remove(
        &self,
        torrent_id: TorrentId,
    ) -> transmission_rpc::types::Result<()> {
        let RpcResponse { .. } = self
            .client
            .torrent_remove(vec![Id::Id(*torrent_id)], true)
            .await?;

        Ok(())
    }
}
