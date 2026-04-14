# Tautulli API Coverage

**Last updated:** 2026-04-14
**Source spec:** docs/api-specs/tautulli.md
**Format:** hand-scraped reference

## Implementation Status

The Tautulli service is **fully onboarded** with a comprehensive SDK covering the most
commonly used Tautulli API commands. The shared dispatch layer
(`crates/lab/src/dispatch/tautulli/`) is fully implemented with catalog, client, params, and
dispatch modules.

Tautulli uses a single command-dispatch API: all requests go to `GET /api/v2` with an
`apikey` query param and a `cmd` param selecting the operation. The SDK's
`TautulliClient::api_get` helper centralises this pattern and unwraps the
`response.data` envelope automatically.

### Surface wiring

| Surface | Status | Notes |
|---------|--------|-------|
| SDK (`lab-apis`) | ✅ (full) | `TautulliClient` in `crates/lab-apis/src/tautulli/client.rs` with 23 methods |
| Dispatch layer | ✅ | `crates/lab/src/dispatch/tautulli/` — catalog, client, params, dispatch |
| MCP | ✅ | `crates/lab/src/mcp/services/tautulli.rs` — thin bridge to dispatch layer |
| CLI | ✅ | `crates/lab/src/cli/tautulli.rs` — generic action dispatcher calling MCP dispatch |
| API | ✅ | `crates/lab/src/api/services/tautulli.rs` — axum route calling dispatch layer |

All surfaces are fully wired and functional.

### Implemented Actions

All actions call `GET /api/v2` with the corresponding `cmd` value. The dispatch layer
handles two built-in actions (`help` and `schema`) automatically before routing to
service-specific logic.

| Action | SDK Method | Tautulli cmd | Destructive |
|--------|-----------|-------------|-------------|
| `activity.list` | `get_activity()` | `get_activity` | No |
| `activity.stream` | `get_stream_data(session_key)` | `get_stream_data` | No |
| `history.list` | `get_history(query)` | `get_history` | No |
| `users.list` | `get_users()` | `get_users` | No |
| `users.get` | `get_user(user_id)` | `get_user` | No |
| `users.watch_time` | `get_user_watch_time_stats(user_id)` | `get_user_watch_time_stats` | No |
| `users.player_stats` | `get_user_player_stats(user_id)` | `get_user_player_stats` | No |
| `libraries.list` | `get_libraries()` | `get_libraries` | No |
| `libraries.get` | `get_library(section_id)` | `get_library` | No |
| `libraries.media_info` | `get_library_media_info(section_id)` | `get_library_media_info` | No |
| `stats.home` | `get_home_stats(time_range, stats_count)` | `get_home_stats` | No |
| `stats.plays_by_date` | `get_plays_by_date(time_range, y_axis)` | `get_plays_by_date` | No |
| `media.recently-added` | `get_recently_added(count, section_id)` | `get_recently_added` | No |
| `media.metadata` | `get_metadata(rating_key)` | `get_metadata` | No |
| `media.children` | `get_children_metadata(rating_key)` | `get_children_metadata` | No |
| `media.export-metadata` | `export_metadata(rating_key, media_type)` | `export_metadata` | No |
| `user.item-stats` | `get_item_user_stats(rating_key, media_type)` | `get_item_user_stats` | No |
| `user.delete-history` | `delete_all_user_history(user_id)` | `delete_all_user_history` | **Yes** |
| `plays.by-day` | `get_plays_by_dayofweek(time_range)` | `get_plays_by_dayofweek` | No |
| `plays.by-hour` | `get_plays_by_hourofday(time_range)` | `get_plays_by_hourofday` | No |
| `plays.by-stream-type` | `get_plays_by_stream_type(time_range)` | `get_plays_by_stream_type` | No |
| `plays.by-month` | `get_plays_per_month(time_range)` | `get_plays_per_month` | No |
| `server.pms-update` | `get_pms_update()` | `get_pms_update` | No |
| `system.info` | `get_server_info()` | `get_server_info` | No |
| `system.settings` | `get_settings()` | `get_settings` | No |

`user.delete-history` is the only destructive action — it requires explicit confirmation
via `"confirm": true` in the API request, `-y` flag in CLI, or MCP elicitation.

### Action Parameters

**`activity.stream`** — required: `session_key` (string, Plex session key).

**`history.list`** — all optional: `page` (integer, 1-based), `page_size` (integer, default
25), `order_dir` (string: asc|desc), `media_type` (string: movie|episode|track),
`user_id` (integer), `section_id` (integer), `rating_key` (integer).

**`users.get`** — required: `user_id` (integer).

**`users.watch_time`** — required: `user_id` (integer).

**`users.player_stats`** — required: `user_id` (integer).

**`libraries.get`** — required: `section_id` (integer).

**`libraries.media_info`** — required: `section_id` (integer).

**`stats.home`** — all optional: `time_range` (integer, days, default 30), `stats_count`
(integer, default 5).

**`stats.plays_by_date`** — all optional: `time_range` (integer, days, default 30), `y_axis`
(string: plays|duration).

**`media.recently-added`** — all optional: `count` (integer, default 5), `section_id`
(string, Plex library section ID).

**`media.metadata`** — required: `rating_key` (string, Plex media item ID).

**`media.children`** — required: `rating_key` (string, Plex media item ID).

**`media.export-metadata`** — required: `rating_key` (string), `media_type` (string: movie,
show, season, episode, artist, album, track).

**`user.item-stats`** — required: `rating_key` (string); optional: `media_type` (string).

**`user.delete-history`** — required: `user_id` (integer). Destructive — deletes all history
for the specified user permanently.

**`plays.by-day`** — optional: `time_range` (integer, days, default 30).

**`plays.by-hour`** — optional: `time_range` (integer, days, default 30).

**`plays.by-stream-type`** — optional: `time_range` (integer, days, default 30).

**`plays.by-month`** — optional: `time_range` (integer, months, default 30).

**`schema`** — required: `action` (string).

### Return Types

All SDK methods return `serde_json::Value` (the unwrapped `response.data` field from
the Tautulli API envelope). Play analytics methods (`plays.*`) return chart data arrays
from the Tautulli API.

## Section Inventory

The table below lists known Tautulli API commands and their implementation status
across all surfaces. The "Impl" column shows SDK (`lab-apis`) coverage. All implemented
actions are available on MCP, CLI, and API surfaces.

| Tautulli cmd | Impl | Action | MCP | CLI | API |
|-------------|------|--------|-----|-----|-----|
| `get_activity` | ✅ | `activity.list` | ✅ | ✅ | ✅ |
| `get_stream_data` | ✅ | `activity.stream` | ✅ | ✅ | ✅ |
| `get_history` | ✅ | `history.list` | ✅ | ✅ | ✅ |
| `get_home_stats` | ✅ | `stats.home` | ✅ | ✅ | ✅ |
| `get_plays_by_date` | ✅ | `stats.plays_by_date` | ✅ | ✅ | ✅ |
| `get_users` | ✅ | `users.list` | ✅ | ✅ | ✅ |
| `get_user` | ✅ | `users.get` | ✅ | ✅ | ✅ |
| `get_user_watch_time_stats` | ✅ | `users.watch_time` | ✅ | ✅ | ✅ |
| `get_user_player_stats` | ✅ | `users.player_stats` | ✅ | ✅ | ✅ |
| `get_libraries` | ✅ | `libraries.list` | ✅ | ✅ | ✅ |
| `get_library` | ✅ | `libraries.get` | ✅ | ✅ | ✅ |
| `get_library_media_info` | ✅ | `libraries.media_info` | ✅ | ✅ | ✅ |
| `get_server_info` | ✅ | `system.info` | ✅ | ✅ | ✅ |
| `get_settings` | ✅ | `system.settings` | ✅ | ✅ | ✅ |
| `get_recently_added` | ✅ | `media.recently-added` | ✅ | ✅ | ✅ |
| `get_metadata` | ✅ | `media.metadata` | ✅ | ✅ | ✅ |
| `get_children_metadata` | ✅ | `media.children` | ✅ | ✅ | ✅ |
| `export_metadata` | ✅ | `media.export-metadata` | ✅ | ✅ | ✅ |
| `get_item_user_stats` | ✅ | `user.item-stats` | ✅ | ✅ | ✅ |
| `delete_all_user_history` | ✅ | `user.delete-history` | ✅ | ✅ | ✅ |
| `get_plays_by_dayofweek` | ✅ | `plays.by-day` | ✅ | ✅ | ✅ |
| `get_plays_by_hourofday` | ✅ | `plays.by-hour` | ✅ | ✅ | ✅ |
| `get_plays_by_stream_type` | ✅ | `plays.by-stream-type` | ✅ | ✅ | ✅ |
| `get_plays_per_month` | ✅ | `plays.by-month` | ✅ | ✅ | ✅ |
| `get_pms_update` | ✅ | `server.pms-update` | ✅ | ✅ | ✅ |
| `get_item_watch_time_stats` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_library_user_stats` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_library_watch_time_stats` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_libraries_table` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_library_names` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `edit_library` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_library` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_all_library_history` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_collections_table` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_new_rating_keys` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_old_rating_keys` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `edit_user` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_user` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_plays_by_source_resolution` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_plays_by_stream_resolution` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_plays_by_top_10_platforms` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_plays_by_top_10_users` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_concurrent_streams_by_stream_type` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_stream_type_by_top_10_platforms` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_stream_type_by_top_10_users` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_notifiers` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_notifier_config` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `add_notifier_config` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_notifier` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_notification_log` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_notification_log` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_notifier_parameters` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_newsletters` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_newsletter_config` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `add_newsletter_config` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_newsletter` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_newsletter_log` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_exports_table` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `download_export` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_export` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_export_fields` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_playlists_table` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_synced_items` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_synced_item` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_mobile_device` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_server_identity` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_server_friendly_name` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_server_id` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_servers_info` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_server_list` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_server_pref` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_logs` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_plex_log` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `download_log` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `download_plex_log` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_login_log` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `backup_config` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `backup_db` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `download_config` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `download_database` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_cache` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_image_cache` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_recently_added` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_temp_sessions` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_media_info_cache` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_history` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_hosted_images` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `delete_lookup_info` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_geoip_lookup` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `docs` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `docs_md` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `get_date_formats` | ⬜ | - | ⬜ | ⬜ | ⬜ |
| `arnold` | ⬜ | - | ⬜ | ⬜ | ⬜ |

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented |
| ⬜ | Not implemented yet |
| - | Not applicable |
