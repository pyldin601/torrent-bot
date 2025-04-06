use crate::transmission::{TorrentId, TransmissionClientError};
use transmission_rpc::types::{Id, Torrent};

impl From<&TorrentId> for Id {
    fn from(value: &TorrentId) -> Self {
        match value {
            TorrentId::Id(id) => Id::Id(*id),
            TorrentId::Hash(hash) => Id::Hash(hash.clone()),
        }
    }
}

impl TryInto<TorrentId> for Torrent {
    type Error = TransmissionClientError;

    fn try_into(self) -> Result<TorrentId, Self::Error> {
        self.hash_string
            .map(TorrentId::Hash)
            .ok_or(TransmissionClientError::MissingHashString)
    }
}
