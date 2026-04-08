# Tautulli API Coverage

**Last updated:** 2026-04-08
**Source spec:** docs/api-specs/tautulli.md
**Format:** hand-scraped reference

## Summary

- This doc is a lightweight implementation aid, not a machine-generated contract.
- The source file remains the canonical endpoint reference for this service.

## Section Inventory

| Section | Key Operations |
|---------|----------------|
| Activity & Streaming | get_activity, get_stream_data |
| History & Statistics | get_history, get_home_stats, get_item_user_stats, get_item_watch_time_stats, get_library_user_stats, get_library_watch_time_stats |
| Graph Data | get_plays_by_date, get_plays_by_dayofweek, get_plays_by_hourofday, get_plays_by_source_resolution, get_plays_by_stream_resolution, get_plays_by_stream_type, get_plays_by_top_10_platforms, get_plays_by_top_10_users, get_plays_per_month, get_concurrent_streams_by_stream_type, get_stream_type_by_top_10_platforms, get_stream_type_by_top_10_users |
| Libraries | get_libraries, get_libraries_table, get_library, get_library_media_info, get_library_names, edit_library, delete_library, delete_all_library_history, get_collections_table |
| Metadata | get_metadata, get_children_metadata, get_recently_added, get_new_rating_keys, get_old_rating_keys |
| Users | get_user, edit_user, delete_user, delete_all_user_history |
| Notifications & Newsletters | get_notifiers, get_notifier_config, add_notifier_config, delete_notifier, get_notification_log, delete_notification_log, get_notifier_parameters, get_newsletters, get_newsletter_config, add_newsletter_config, delete_newsletter, get_newsletter_log (+1 more) |
| Exports | export_metadata, get_exports_table, download_export, delete_export, get_export_fields |
| Playlists & Sync | get_playlists_table, get_synced_items, delete_synced_item, delete_mobile_device |
| Server Information | get_server_info, get_server_identity, get_server_friendly_name, get_server_id, get_servers_info, get_server_list, get_server_pref, get_pms_update |
| Logging | get_logs, get_plex_log, download_log, download_plex_log, delete_login_log |
| Settings & Configuration | get_settings, backup_config, backup_db, download_config, download_database |
| Cache & Database Maintenance | delete_cache, delete_image_cache, delete_recently_added, delete_temp_sessions, delete_media_info_cache, delete_history |
| Third-Party Integrations | delete_hosted_images, delete_lookup_info, get_geoip_lookup |
| Miscellaneous | docs, docs_md, get_date_formats, arnold |

## Notes

- This service starts with an implementation status of not started across CLI, API, and MCP.
- Expand this document into a full matrix when service work begins.
