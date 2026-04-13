# SABnzbd API Coverage

**Last updated:** 2026-04-10
**Source spec:** docs/upstream-api/sabnzbd.md
**Format:** hand-scraped reference

## Summary

- This doc is a lightweight implementation aid, not a machine-generated contract.
- The source file remains the canonical endpoint reference for this service.

## Section Inventory

| Section | Key Operations |
|---------|----------------|
| Queue | queue, pause, resume, speedlimit, change_complete_action, sort, addurl, addfile, addlocalfile, delete, purge, switch (+8 more) |
| History | history, retry, retry_all, delete, mark_as_completed |
| Status | status, fullstatus, unblock_server, delete_orphan, add_orphan |
| Configuration | get_config, set_config, del_config, set_config_default |
| Information | version, auth, get_cats, get_scripts, server_stats, warnings, translate, showlog |
| System Control | See source file |

## Notes

- Dispatch-layer migration is in place for implemented actions:
  - catalog + execution in `crates/lab/src/dispatch/sabnzbd/`
  - CLI shim in `crates/lab/src/cli/sabnzbd.rs`
  - MCP shim in `crates/lab/src/mcp/services/sabnzbd.rs`
  - API shim in `crates/lab/src/api/services/sabnzbd.rs`
- Current dispatch actions implemented end-to-end:
  - `help`
  - `server.version`
  - `queue.list`
  - `queue.delete` (destructive)
  - `history.list`
  - `history.delete` (destructive)
  - `history.purge` (destructive)
  - `server.stats`
  - `server.warnings`
  - `queue.pause`
  - `queue.resume`
  - `queue.speed.limit`
- Remaining SABnzbd upstream modes are still pending implementation.
