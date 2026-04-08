# SABnzbd API Coverage

**Last updated:** 2026-04-08
**Source spec:** docs/api-specs/sabnzbd.md
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

- This service starts with an implementation status of not started across CLI, API, and MCP.
- Expand this document into a full matrix when service work begins.
