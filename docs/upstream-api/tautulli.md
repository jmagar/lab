# Tautulli API Reference

> Source: https://github.com/Tautulli/Tautulli/wiki/Tautulli-API-Reference
> Style: flat RPC over `?cmd=` query string. All requests require `apikey=APIKEY`.
> Base URL: `http://host:port/api/v2?apikey=APIKEY&cmd=COMMAND&[params]`

## Activity & Streaming

**get_activity**
- Parameters: `session_key` (int, optional), `session_id` (str, optional)
- Returns: Current PMS activity with detailed session information

**get_stream_data**
- Parameters: `row_id` (int) OR `session_key` (int); both required
- Returns: Stream codec, bitrate, resolution, and transcoding details

## History & Statistics

**get_history**
- Parameters: Multiple optional filters (user, date range, media_type, transcode_decision, etc.)
- Returns: Paginated history records with play counts and durations

**get_home_stats**
- Parameters: `time_range` (int), `stats_type` (str), optional filters
- Returns: Top movies, TV, music, users, platforms, and concurrent statistics

**get_item_user_stats**
- Parameters: `rating_key` (str, required); `media_type` (str, optional)
- Returns: Per-user play counts and total watch time for specific media

**get_item_watch_time_stats**
- Parameters: `rating_key` (str, required); `query_days` (str, optional)
- Returns: Watch time aggregated by configurable day ranges

**get_library_user_stats**
- Parameters: `section_id` (str, required); `grouping` (int, optional)
- Returns: User statistics for a specific library section

**get_library_watch_time_stats**
- Parameters: `section_id` (str, required); `query_days` (str, optional)
- Returns: Library-wide watch time by day range

## Graph Data

**get_plays_by_date**
- Parameters: `time_range`, `y_axis` ("plays"/"duration"), optional user filter
- Returns: Time-series data by library type

**get_plays_by_dayofweek** / **get_plays_by_hourofday**
- Parameters: `time_range`, `y_axis`, optional filters
- Returns: Aggregated stats by day of week / hour

**get_plays_by_source_resolution** / **get_plays_by_stream_resolution**
- Parameters: `time_range`, `y_axis`, optional filters
- Returns: Plays grouped by original/streamed resolution

**get_plays_by_stream_type**
- Parameters: `time_range`, `y_axis`, optional filters
- Returns: Direct Play/Stream/Transcode breakdown by date

**get_plays_by_top_10_platforms** / **get_plays_by_top_10_users**
- Parameters: `time_range`, `y_axis`, optional filters
- Returns: Stats for top 10 platforms / users

**get_plays_per_month**
- Parameters: `time_range`, `y_axis`, optional filters
- Returns: Monthly aggregated statistics

**get_concurrent_streams_by_stream_type**
- Parameters: `time_range` (str, optional), `user_id` (str, optional)
- Returns: Direct Play/Stream/Transcode concurrent counts with maximum

**get_stream_type_by_top_10_platforms** / **get_stream_type_by_top_10_users**
- Parameters: `time_range`, `y_axis`, optional filters
- Returns: Stream type distribution by platform / user

## Libraries

**get_libraries** — list of all library sections with metadata
**get_libraries_table** — paginated library data with stats; filters: order_column, search, pagination
**get_library** — `section_id` (req), `include_last_accessed` (opt) → single library details
**get_library_media_info** — `section_id` OR `rating_key`; paginated media file info (codecs, bitrate, size)
**get_library_names** — lightweight list of section IDs and names
**edit_library** — `section_id`, `custom_thumb`, `custom_art`, `keep_history` (all req)
**delete_library** — `section_id` (req); `row_ids` (opt)
**delete_all_library_history** — `server_id`, `section_id` (req); `row_ids` (opt)
**get_collections_table** — `section_id` (req) → paginated collection data

## Metadata

**get_metadata** — `rating_key` (str) OR `sync_id` (str) → comprehensive metadata
**get_children_metadata** — `rating_key`, `media_type` (req) → child items (seasons/episodes)
**get_recently_added** — `count` (req); `start`, `media_type`, `section_id` (opt)
**get_new_rating_keys** / **get_old_rating_keys** — `rating_key`, `media_type` (req)

## Users

**get_user** — returns user information
**edit_user** — `user_id`, `friendly_name`, `custom_thumb`, `keep_history`, `allow_guest` (all req)
**delete_user** — `user_id` (req); `row_ids` (opt)
**delete_all_user_history** — `user_id` (req); `row_ids` (opt)

## Notifications & Newsletters

**get_notifiers** / **get_notifier_config** / **add_notifier_config** / **delete_notifier**
**get_notification_log** / **delete_notification_log** / **get_notifier_parameters**
**get_newsletters** / **get_newsletter_config** / **add_newsletter_config** / **delete_newsletter**
**get_newsletter_log** / **delete_newsletter_log**

## Exports

**export_metadata** — `section_id`/`user_id`/`rating_key` (one req)
**get_exports_table** — paginated export records
**download_export** — `export_id` (req)
**delete_export** — `export_id` OR `delete_all`
**get_export_fields** — `media_type` (req); `sub_media_type` (opt)

## Playlists & Sync

**get_playlists_table** — `section_id` OR `user_id`
**get_synced_items** — optional `machine_id`, `user_id`
**delete_synced_item** — `client_id`, `sync_id` (req)
**delete_mobile_device** — `mobile_device_id` OR `device_id`

## Server Information

**get_server_info** — PMS connection details and version
**get_server_identity** — machine identifier and version
**get_server_friendly_name** — string server name
**get_server_id** — `hostname`, `port` (req); `ssl`, `remote` (opt)
**get_servers_info** — list of accessible servers
**get_server_list** — published servers from Plex.tv
**get_server_pref** — `pref` (req)
**get_pms_update** — available PMS update details

## Logging

**get_logs** — optional `sort`, `search`, `regex`, `start`, `end`
**get_plex_log** — optional `window`, `logfile`
**download_log** — optional `logfile` ("tautulli"/"tautulli_api"/"plex_websocket")
**download_plex_log** — optional `logfile`
**delete_login_log**

## Settings & Configuration

**get_settings** — optional `key`
**backup_config** / **backup_db**
**download_config** / **download_database**

## Cache & Database Maintenance

**delete_cache** / **delete_image_cache** / **delete_recently_added** / **delete_temp_sessions**
**delete_media_info_cache** — `section_id` (req)
**delete_history** — `row_ids` (req)

## Third-Party Integrations

**delete_hosted_images** — optional `rating_key`, `service` (`imgur`/`cloudinary`), `delete_all`
**delete_lookup_info** — optional `rating_key`, `service` (`themoviedb`/`tvmaze`/`musicbrainz`), `delete_all`
**get_geoip_lookup** — `ip_address` (req)

## Miscellaneous

**docs** — API command reference as dictionary
**docs_md** — markdown-formatted API documentation
**get_date_formats** — JSON with configured date and time formats
**arnold** — easter egg
