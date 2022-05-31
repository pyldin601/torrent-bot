# torrent-bot
Utility, that syncs topics tracked by user in toloka torrent tracker with transmission torrent client.

## Environment variables example
```dotenv
# Path to the storage where's the sync state is located
STORAGE_FILE=torrent-bot.db
# Creadentials used to login to the toloka torrent tracker
TOLOKA_USERNAME=username
TOLOKA_PASSWORD=password
# Credentials to connect to the transmission client
TRANS_URL=http://192.168.1.78:9091/transmission/rpc
TRANS_USERNAME=hello
TRANS_PASSWORD=world
# Directory, where the torrent files should be downloaded to
TRANS_DOWNLOAD_DIRECTORY=/data
```
