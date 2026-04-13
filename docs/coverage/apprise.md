# Apprise API Coverage

**Last updated:** 2026-04-12
**Source spec:** docs/upstream-api/apprise.md
**Format:** hand-scraped reference

## Summary

- This doc is a lightweight implementation aid, not a machine-generated contract.
- The source file remains the canonical endpoint reference for this service.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and wired through SDK, dispatch, CLI, MCP, and API |
| ⬜ | Not implemented yet; rows are planning inventory only |

## API Inventory

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /status | health | ✅ | ✅ | ✅ | ✅ |
| POST | /notify | notify | ✅ | ✅ | ✅ | ✅ |
| POST | /notify/{KEY} | notify_key | ✅ | ✅ | ✅ | ✅ |
| POST | /add/{KEY} | add_config | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /get/{KEY} | get_config | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /del/{KEY} | delete_config | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /json/urls/{KEY} | get_urls | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /details | details | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /metrics | metrics | ⬜ | ⬜ | ⬜ | ⬜ |

## Notes

- The first online slice is `server.health`, `notify.send`, and `notify.key.send`.
- Stateful config-management and details/metrics endpoints are still pending.
