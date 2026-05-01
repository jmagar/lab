# NotebookLM Coverage

`notebooklm` is implemented from the public `teng-lin/notebooklm-py` project's
documented Python API and reverse-engineered batchexecute RPC contract. Google
NotebookLM does not publish an official API, so this integration should be
treated as best-effort and more drift-prone than OpenAPI-backed services.

## Auth

The service reads Playwright `storage_state.json` cookies produced by
`notebooklm-py`'s login flow or compatible browser automation.

Supported configuration:

- `NOTEBOOKLM_AUTH_JSON` — inline Playwright storage JSON
- `NOTEBOOKLM_STORAGE` — explicit path to `storage_state.json`
- `NOTEBOOKLM_HOME` plus `NOTEBOOKLM_PROFILE` — profile-aware default path

The client fetches NotebookLM's homepage at call time to extract the current
`SNlM0e` CSRF token and `FdrFJe` session id, matching the upstream Python
project's flow.

## Implemented Actions

| Action | Status | Notes |
|--------|--------|-------|
| `help` | implemented | Shared action catalog |
| `schema` | implemented | Per-action schema |
| `notebook.list` | implemented | `wXbhsf` |
| `notebook.create` | implemented | `CCqFvf` |
| `notebook.get` | implemented | `rLM1Ne` |
| `notebook.delete` | implemented | `WWINqb`; destructive |
| `source.list` | implemented | Uses notebook payload from `rLM1Ne` |
| `source.add_url` | implemented | `izAoDd`; regular URL source shape |
| `server.health` | implemented | Lists notebooks as authenticated probe |

## Not Yet Implemented

- Browser login orchestration
- File, pasted text, YouTube-specialized, and Google Drive source add paths
- Chat, conversation history, notes, research, sharing, settings, artifacts,
  generation, downloads, and export
- Automatic retry after token refresh failure
- Live smoke tests against a real NotebookLM account

## Verification

Use:

```bash
cargo check -p lab@0.12.1 --all-features
cargo test -p lab@0.12.1 --all-features dispatch::notebooklm
```

Live smoke tests require valid NotebookLM cookies:

```bash
lab notebooklm notebook.list --json
lab notebooklm notebook.create --params '{"title":"Lab smoke test"}' --json
```
