# Navidrome Coverage

Status: first-class v1 read-only Subsonic/OpenSubsonic surface.

Actions: `server.ping`, `artist.list`, `album.list`, `album.get`, `search.query`, `playlist.list`, plus built-in `help` and `schema`.

Deferred: private web UI API, streaming, downloads, cover art, scan start, scrobble, ratings, stars, playlist mutation, shares, radio mutation.

Security: `NAVIDROME_TOKEN` and `NAVIDROME_SALT` are secret. Lab requires precomputed token+salt auth and does not send cleartext `p` password auth.

