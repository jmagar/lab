# qBittorrent WebUI API v5.0 Reference

> Sources:
> - Official wiki: https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)
> - Python wrapper (canonical type/method reference): https://qbittorrent-api.readthedocs.io/en/latest/api.html
>
> Style: REST. Auth via cookie session (login → SID cookie).
> Base URL: `http://host:port/api/v2/<endpoint>`

## Authentication

| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/login` | Authenticate with username/password — returns SID cookie |
| POST | `/auth/logout` | Terminate session |

## Application

| Method | Path | Description |
|--------|------|-------------|
| GET  | `/app/version` | App version string |
| GET  | `/app/webapiVersion` | WebAPI version |
| GET  | `/app/buildInfo` | Qt, libtorrent, OpenSSL versions |
| POST | `/app/shutdown` | Terminate application |
| GET  | `/app/preferences` | All user configuration |
| POST | `/app/setPreferences` | Update preferences (JSON) |
| GET  | `/app/defaultSavePath` | Default download directory |
| GET  | `/app/cookies` | HTTP cookies for torrent downloads |
| POST | `/app/setCookies` | Configure cookies |

## Log

| Method | Path | Description |
|--------|------|-------------|
| GET | `/log/main` | Application logs (filterable by type) |
| GET | `/log/peers` | Peer blocking events |

## Sync

| Method | Path | Description |
|--------|------|-------------|
| GET | `/sync/maindata?rid=N` | Torrent + global state changes since last rid |
| GET | `/sync/torrentPeers?hash=H&rid=N` | Peer data for torrent |

## Transfer Info

| Method | Path | Description |
|--------|------|-------------|
| GET  | `/transfer/info` | Bandwidth speeds and connection status |
| GET  | `/transfer/speedLimitsMode` | Alt speed limits enabled? |
| POST | `/transfer/toggleSpeedLimitsMode` | Toggle alt speed limits |
| GET  | `/transfer/downloadLimit` | Global download limit (B/s) |
| POST | `/transfer/setDownloadLimit` | Set global download limit |
| GET  | `/transfer/uploadLimit` | Global upload limit (B/s) |
| POST | `/transfer/setUploadLimit` | Set global upload limit |
| POST | `/transfer/banPeers` | Ban peers (`peers=` pipe-separated host:port) |

## Torrent Management — Listing & Properties

| Method | Path | Params | Description |
|--------|------|--------|-------------|
| GET | `/torrents/info` | filter, category, tag, sort, limit, offset | List torrents |
| GET | `/torrents/properties` | hash | Detailed torrent properties |
| GET | `/torrents/trackers` | hash | Tracker list |
| GET | `/torrents/webseeds` | hash | Web seed URLs |
| GET | `/torrents/files` | hash, indexes? | File listing with priorities |
| GET | `/torrents/pieceStates` | hash | State of each piece |
| GET | `/torrents/pieceHashes` | hash | SHA1 of all pieces |

## Torrent Management — Control

| Method | Path | Params | Description |
|--------|------|--------|-------------|
| POST | `/torrents/stop` | hashes (pipe-sep or `all`) | Pause torrents |
| POST | `/torrents/start` | hashes (pipe-sep or `all`) | Resume torrents |
| POST | `/torrents/delete` | hashes, deleteFiles (bool) | Remove torrents |
| POST | `/torrents/recheck` | hashes | Verify integrity |
| POST | `/torrents/reannounce` | hashes | Contact trackers immediately |

## Torrent Management — Add

**POST `/torrents/add`** — multipart with options:
`urls` (newline-separated), `torrents` (file uploads), `savepath`, `category`,
`tags`, `paused`, `root_folder`, `skip_checking`, `rename`, `upLimit`,
`dlLimit`, `ratioLimit`, `seedingTimeLimit`, `autoTMM`, `sequentialDownload`,
`firstLastPiecePrio`, `cookie`

## Torrent Management — Trackers

| Method | Path | Params |
|--------|------|--------|
| POST | `/torrents/addTrackers` | hash, urls (newline-sep) |
| POST | `/torrents/editTracker` | hash, origUrl, newUrl |
| POST | `/torrents/removeTrackers` | hash, urls (pipe-sep) |
| POST | `/torrents/addPeers` | hashes, peers (pipe-sep host:port) |

## Torrent Management — Priority

| Method | Path | Params |
|--------|------|--------|
| POST | `/torrents/increasePrio` | hashes |
| POST | `/torrents/decreasePrio` | hashes |
| POST | `/torrents/topPrio` | hashes |
| POST | `/torrents/bottomPrio` | hashes |
| POST | `/torrents/filePrio` | hash, id (pipe-sep), priority (0-7) |

## Torrent Management — Speed Limits

| Method | Path | Params |
|--------|------|--------|
| POST | `/torrents/downloadLimit` | hashes |
| POST | `/torrents/setDownloadLimit` | hashes, limit (B/s) |
| POST | `/torrents/uploadLimit` | hashes |
| POST | `/torrents/setUploadLimit` | hashes, limit (B/s) |
| POST | `/torrents/setShareLimits` | hashes, ratioLimit, seedingTimeLimit, inactiveSeedingTimeLimit |

## Torrent Management — Organization

| Method | Path | Params |
|--------|------|--------|
| POST | `/torrents/setLocation` | hashes, location |
| POST | `/torrents/rename` | hash, name |
| POST | `/torrents/setCategory` | hashes, category |
| POST | `/torrents/addTags` | hashes, tags (csv) |
| POST | `/torrents/removeTags` | hashes, tags (csv) |

## Categories & Tags

| Method | Path | Params |
|--------|------|--------|
| GET  | `/torrents/categories` | — |
| POST | `/torrents/createCategory` | category, savePath |
| POST | `/torrents/editCategory` | category, savePath |
| POST | `/torrents/removeCategories` | categories (newline-sep) |
| GET  | `/torrents/tags` | — |
| POST | `/torrents/createTags` | tags (csv) |
| POST | `/torrents/deleteTags` | tags (csv) |

## Torrent Management — Advanced

| Method | Path | Params |
|--------|------|--------|
| POST | `/torrents/setAutoManagement` | hashes, enable (bool) |
| POST | `/torrents/toggleSequentialDownload` | hashes |
| POST | `/torrents/toggleFirstLastPiecePrio` | hashes |
| POST | `/torrents/setForceStart` | hashes, value (bool) |
| POST | `/torrents/setSuperSeeding` | hashes, value (bool) |
| POST | `/torrents/renameFile` | hash, oldPath, newPath |
| POST | `/torrents/renameFolder` | hash, oldPath, newPath |

## RSS (Experimental)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/rss/addFolder` | Create feed folder |
| POST | `/rss/addFeed` | Subscribe to RSS |
| POST | `/rss/removeItem` | Delete feed/folder |
| POST | `/rss/moveItem` | Move feed |
| GET  | `/rss/items` | All feed articles |
| POST | `/rss/markAsRead` | Mark articles read |
| POST | `/rss/refreshItem` | Update feed |
| POST | `/rss/setRule` | Auto-download rule |
| POST | `/rss/renameRule` | Rename rule |
| POST | `/rss/removeRule` | Delete rule |
| GET  | `/rss/rules` | List rules |
| GET  | `/rss/matchingArticles` | Articles matching rule |

## Search

| Method | Path | Params |
|--------|------|--------|
| POST | `/search/start` | pattern, plugins?, category? |
| POST | `/search/stop` | id |
| GET  | `/search/status` | — |
| GET  | `/search/results` | id, limit?, offset? |
| POST | `/search/delete` | id |
| GET  | `/search/plugins` | — |
| POST | `/search/installPlugin` | sources (URLs) |
| POST | `/search/uninstallPlugin` | names (pipe-sep) |
| POST | `/search/enablePlugin` | names, enable (bool) |
| POST | `/search/updatePlugins` | — |
