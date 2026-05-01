# Navidrome Source Contract

Retrieved: 2026-05-01

Sources:
- https://www.navidrome.org/docs/developers/subsonic-api/
- https://www.subsonic.org/pages/api.jsp
- https://opensubsonic.netlify.app/docs/

Navidrome supports Subsonic API v1.16.1 with documented exceptions and OpenSubsonic extensions. Lab uses only the supported `/rest/*.view` compatibility API, not private web UI APIs.

## Auth

Lab requires `NAVIDROME_URL`, `NAVIDROME_USERNAME`, `NAVIDROME_TOKEN`, and `NAVIDROME_SALT`. Cleartext `p` password auth is not used. Query parameters `u`, `t`, `s`, `p`, and `c` are secret-bearing and must be redacted from logs and errors.

## V1 Actions

| Action | Endpoint | Bounds | Hosted posture |
|---|---|---|---|
| `server.ping` | `ping.view` | no body | safe |
| `artist.list` | `getArtists.view` | upstream bounded | safe |
| `album.list` | `getAlbumList2.view` | `size` 1-100, explicit `offset` | safe |
| `album.get` | `getAlbum.view` | one string id | safe |
| `search.query` | `search3.view` | `size` 1-100 per class, explicit `offset` | safe |
| `playlist.list` | `getPlaylists.view` | upstream bounded | safe |

IDs are strings and must not be parsed as integers.

## Deferred

Streaming, downloading, cover art, shares, stars, ratings, scrobbling, playlist mutation, scans, private `/api` routes, and radio mutation are deferred.

