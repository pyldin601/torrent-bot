use crate::task_db;
use torrent_bot_clients::transmission;

impl Into<task_db::TorrentId> for &transmission::TorrentId {
    fn into(self) -> task_db::TorrentId {
        match self {
            transmission::TorrentId::Id(id) => task_db::TorrentId::Id(*id),
            transmission::TorrentId::Hash(hash) => task_db::TorrentId::Hash(hash.to_string()),
        }
    }
}

impl Into<transmission::TorrentId> for &task_db::TorrentId {
    fn into(self) -> transmission::TorrentId {
        match self {
            task_db::TorrentId::Id(id) => transmission::TorrentId::Id(*id),
            task_db::TorrentId::Hash(hash) => transmission::TorrentId::Hash(hash.to_string()),
        }
    }
}
