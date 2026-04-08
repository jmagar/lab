# SABnzbd API Reference

> Source: https://sabnzbd.org/wiki/configuration/4.5/api
> Style: flat RPC over `?mode=` query string. All requests require `apikey=APIKEY` (except `version`/`auth`).
> Base URL: `http://host:port/api?apikey=APIKEY&mode=MODE&output=json&[params]`
> Output format: `output=json` (default) or `output=xml`

## Queue

**queue** — Retrieve full queue with job details
- `start` (int, opt), `limit` (int, opt), `cat` (str, opt), `priority` (int, opt)
- `status` (str, opt), `search` (str, opt), `nzo_ids` (csv, opt)
- Returns: queue status, speed, disk space, array of job slots

**pause** / **resume** — Pause/resume entire queue (no params)

**speedlimit** — `value` (str/int, req): % of max or numeric with K/M

**change_complete_action** — `value` (req): `hibernate_pc`, `standby_pc`, `shutdown_program`, `script_NAME`

**sort** — `sort` (`avg_age`|`name`|`remaining`|`size`), `dir` (`asc`|`desc`)

**addurl** — Add NZB via URL
- `name` (req): URL-encoded NZB link
- `nzbname`, `password`, `cat`, `script`, `priority`, `pp` (opt)
- Returns: `{ status, nzo_ids }`

**addfile** — Upload NZB via multipart (same params as addurl, file in `name`/`nzbfile`)

**addlocalfile** — Add NZB from server filesystem path (same params as addurl)

### Single-job operations

**pause** / **resume** (single) — `value` (req): nzo_id

**delete** — `value` (req): nzo_id(s) or `all`; `del_files` (opt): 1 to remove files

**purge** — `search` (opt), `del_files` (opt) → array of removed nzb_ids

**switch** — `value` (req): nzo_id, `value2` (req): target id or position

**change_cat** — `value` (req): nzo_id, `value2` (req): new category

**change_script** — `value` (req): nzo_id, `value2` (req): script filename

**priority** — `value` (req): nzo_id, `value2` (req): priority -100..2

**change_opts** — `value` (req): nzo_id, `value2` (req): pp option 0-3

**rename** — `value` (req): nzo_id, `value2` (req): new name or `NAME{{PASSWORD}}`, `value3` (opt): password

**get_files** — `value` (req): nzo_id → `{ files: [{status, size, age, filename, nzf_id}] }`

**move_nzf_bulk** — `name` (`top`|`bottom`|`up`|`down`), `value` (nzo_id), `nzf_ids` (csv), `size` (opt)

**delete_nzf** — `value` (nzo_id), `value2` (csv nzf_ids) → array of removed nzf_ids

## History

**history** — Retrieve completed/failed jobs
- `start`, `limit`, `archive`, `cat`, `status`, `search`, `nzo_ids`, `failed_only`, `last_history_update` (all opt)
- Returns: `{ history, day_size, week_size, month_size, total_size, last_history_update }`

**retry** — `value` (req): nzo_id; `password` (opt); `nzbfile` (opt)

**retry_all** — Retry all failed jobs (no params)

**delete** (history) — `value` (req): nzo_id(s) or `all` or `failed`; `archive` (opt, default 1); `del_files` (opt)

**mark_as_completed** — `value` (req): comma-separated nzo_ids

## Status

**status** / **fullstatus** — Comprehensive system + queue status
- `skip_dashboard` (opt): skip IPv4 lookup (faster)
- `calculate_performance` (opt)
- Returns: local/public IP, CPU, folders, disk space, server connections, active jobs

**unblock_server** — `value` (req): server name

**delete_orphan** — `value` (req): URL-encoded folder name

**add_orphan** — `value` (req): URL-encoded folder name (retry orphan)

## Configuration

**get_config** — `section` (opt): `misc`/`servers`/`categories`/etc.; `keyword` (opt)

**set_config** — `section` (req), `keyword` (req), `value` (req)

**del_config** — `section` (req): `servers`/`rss`/`categories`/`sorters`; `keyword` (req)

**set_config_default** — `keyword` (req, repeatable)

## Information

**version** — Get running version (no auth required)

**auth** — Get available authentication methods (no auth required)

**get_cats** — List all categories

**get_scripts** — List all scripts

**server_stats** — Download stats by timeframe and server

**warnings** — All active system warnings

**translate** — `value` (req): text to localize

**showlog** — Anonymized log file with config copy (binary download)

## System Control

| Mode | Purpose |
|------|---------|
| `shutdown` | Stop SABnzbd |
| `restart` | Restart application |
| `restart_repair` | Restart with queue repair |
| `pause_pp` | Pause post-processing |
| `resume_pp` | Resume post-processing |
| `rss_now` | Fetch RSS feeds immediately |
| `watched_now` | Scan watched folder |
| `reset_quota` | Reset bandwidth quota |
