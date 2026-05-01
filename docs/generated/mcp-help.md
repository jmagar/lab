# Lab MCP Help

Generated from `target/debug/lab --json help`, the shared catalog used by `lab://catalog`.

## `extract`

Pull API keys and URLs from existing service config files

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `scan` | Scan an appdata path and return discovered service credentials | false | `uri: json`<br>`redact_secrets: json` | `DiscoveredService[]` |
| `apply` | Scan and write discovered credentials into ~/.lab/.env (with backup) | true | `uri*: json`<br>`services: json`<br>`env_path: json` | `WritePlan` |
| `diff` | Show what 'apply' would change vs the current env file (no writes) | false | `uri*: json` | `WritePlan` |

## `gateway`

Manage proxied upstream MCP gateways

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `gateway.list` | List configured gateways | false | - | `ServerView[]` |
| `gateway.tool_search.get` | Read the gateway-wide tool-search settings | false | - | `ToolSearchConfig` |
| `gateway.tool_search.set` | Enable or disable gateway-wide tool-search mode for all exposed upstream tools | true | `enabled*: json`<br>`top_k_default: json`<br>`max_tools: json` | `ToolSearchConfig` |
| `gateway.server.get` | Get one unified server row by id | false | `id*: json` | `ServerView` |
| `gateway.supported_services` | List metadata-backed Lab services that can be added as virtual servers | false | - | `SupportedServiceView[]` |
| `gateway.virtual_server.enable` | Enable a configured Lab-backed service as a virtual server | true | `id*: json` | `ServerView` |
| `gateway.virtual_server.disable` | Disable a Lab-backed virtual server without removing its config | true | `id*: json` | `ServerView` |
| `gateway.virtual_server.remove` | Remove a Lab-backed virtual server config entry | true | `id*: json` | `ServerView` |
| `gateway.virtual_server.quarantine.list` | List Lab-backed virtual servers quarantined during config migration | false | - | `ServerView[]` |
| `gateway.virtual_server.quarantine.restore` | Restore a quarantined Lab-backed virtual server into the active gateway list | true | `id*: json` | `ServerView` |
| `gateway.virtual_server.set_surface` | Enable or disable one surface on a Lab-backed virtual server | true | `id*: json`<br>`surface*: json`<br>`enabled*: json` | `ServerView` |
| `gateway.virtual_server.get_mcp_policy` | Read the MCP action allowlist for a Lab-backed virtual server | false | `id*: json` | `VirtualServerMcpPolicyView` |
| `gateway.virtual_server.set_mcp_policy` | Replace the MCP action allowlist for a Lab-backed virtual server | true | `id*: json`<br>`allowed_actions*: json` | `VirtualServerMcpPolicyView` |
| `gateway.service_config.get` | Read canonical config for one Lab-backed service | false | `service*: json` | `ServiceConfigView` |
| `gateway.service_actions` | List compiled action metadata for one Lab-backed service | false | `service*: json` | `ServiceActionView[]` |
| `gateway.service_config.set` | Write canonical config for one Lab-backed service | true | `service*: json`<br>`values*: json` | `ServiceConfigView` |
| `gateway.get` | Get one configured gateway | false | `name*: json` | `GatewayView` |
| `gateway.test` | Test a configured or proposed gateway without saving it | false | `name: json`<br>`spec: json`<br>`allow_stdio: json` | `GatewayTestResult` |
| `gateway.add` | Add a gateway and reconcile runtime state | true | `spec*: json`<br>`bearer_token_value: json`<br>`allow_stdio: json` | `GatewayView` |
| `gateway.update` | Update a gateway and reconcile runtime state | true | `name*: json`<br>`patch*: json`<br>`bearer_token_value: json`<br>`allow_stdio: json` | `GatewayView` |
| `gateway.remove` | Remove a gateway and reconcile runtime state | true | `name*: json` | `GatewayView` |
| `gateway.reload` | Reload gateways from config and reconcile runtime state | true | - | `GatewayCatalogDiff` |
| `gateway.status` | Get current runtime gateway status | false | `name: json` | `GatewayRuntimeView[]` |
| `gateway.discovered_tools` | List discovered upstream tool metadata and exposure state for one gateway | false | `name*: json` | `GatewayToolExposureRowView[]` |
| `gateway.discovered_resources` | List discovered upstream resources for one gateway | false | `name*: json` | `string[]` |
| `gateway.discovered_prompts` | List discovered upstream prompts for one gateway | false | `name*: json` | `string[]` |
| `gateway.oauth.probe` | Probe a URL for OAuth support via RFC 8414 AS metadata discovery. Registers a transient OAuth manager so subsequent authorize calls work. | true | `url*: json` | `ProbeResult` |
| `gateway.oauth.start` | Start the upstream OAuth flow for the shared gateway credential and return the browser authorization URL | false | `upstream*: json`<br>`subject: json` | `BeginAuthorization` |
| `gateway.oauth.status` | Read upstream OAuth status for the shared gateway credential | false | `upstream*: json`<br>`subject: json` | `UpstreamOauthStatusView` |
| `gateway.oauth.clear` | Clear stored upstream OAuth credentials for the shared gateway credential | true | `upstream*: json`<br>`subject: json` | `ok` |
| `gateway.mcp.enable` | Enable an upstream MCP server so new sessions discover and proxy it again | true | `name*: json` | `GatewayView` |
| `gateway.mcp.list` | List upstream MCP runtime state, discovery counts, and likely stale process counts | false | - | `GatewayMcpRuntimeView[]` |
| `gateway.mcp.disable` | Disable an upstream MCP server and optionally clean up running processes | true | `name*: json`<br>`cleanup: json`<br>`aggressive: json` | `GatewayView + optional cleanup result` |
| `gateway.mcp.cleanup` | Kill or preview running processes associated with one upstream MCP server | true | `name*: json`<br>`aggressive: json`<br>`dry_run: json` | `GatewayCleanupView` |

## `doctor`

Comprehensive health audit: env vars, system probes, and service reachability

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `system.checks` | Run local system probes: env vars, Docker, disk, ports, config files | false | - | `DoctorReport` |
| `service.probe` | Probe a single named service via its health endpoint | false | `service*: json`<br>`instance: json` | `Finding` |
| `audit.full` | Probe all configured services plus system checks; streams results | false | - | `stream<Finding>` |
| `auth.check` | Check auth/OAuth configuration: env vars, file presence, and Unix file permissions | false | - | `DoctorReport` |

## `setup`

First-run + draft-commit configuration flow

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `state` | First-run + draft snapshot for the wizard / settings UI | false | - | `SetupSnapshot` |
| `schema.get` | UiSchema projection for all (or filtered) services | false | `services: json` | `ServiceSchemaMap` |
| `draft.get` | Read .env.draft with secret values masked to '***' | false | - | `DraftEntry[]` |
| `draft.set` | Write a key (or section) into .env.draft (validated server-side) | false | `entries*: json`<br>`force: json` | `DraftSetOutcome` |
| `draft.commit` | Run audit and atomically merge .env.draft into .env | true | `force: json` | `CommitOutcome` |
| `finalize` | Alias for draft.commit; same params, same returns | true | `force: json` | `CommitOutcome` |

## `logs`

Search and stream local-master runtime logs

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `logs.search` | Search persisted log events with filters | false | `query: json` | `Value` |
| `logs.tail` | Bounded follow-up read from the persisted store | false | `after_ts: json`<br>`since_event_id: json`<br>`limit: json` | `Value` |
| `logs.stats` | Return retention metadata and drop counters | false | - | `Value` |
| `logs.stream` | Live push is HTTP SSE only; dispatch returns guidance | false | - | `Value` |

## `device`

Manage fleet device enrollments

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `enrollments.list` | List pending, approved, and denied node enrollments | false | - | `Value` |
| `enrollments.approve` | Approve a pending node enrollment | true | `node_id*: json`<br>`note: json` | `Value` |
| `enrollments.deny` | Deny a pending or approved node enrollment | true | `node_id*: json`<br>`reason: json` | `Value` |

## `marketplace`

Browse Claude Code/Codex marketplaces, MCP Registry servers, ACP agents, and installable components

Category: `marketplace`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `sources.list` | List configured marketplaces | false | - | `Marketplace[]` |
| `plugins.list` | List plugins across marketplaces. Supports server-side filtering by kind, installed state, and text query. All filter params are optional and additive. | false | `marketplace: json`<br>`kind: json`<br>`installed: json`<br>`query: json` | `Plugin[]` |
| `plugin.get` | Return a single plugin by id (`name@marketplace`) | false | `id*: json` | `Plugin` |
| `plugin.artifacts` | List artifact files shipped with an installed plugin | false | `id*: json` | `Artifact[]` |
| `plugin.workspace` | Load or create an app-managed editable workspace mirror for a plugin | false | `id*: json` | `PluginWorkspace` |
| `plugin.save` | Save a file into the plugin workspace mirror | false | `id*: json`<br>`path*: json`<br>`content*: json` | `SaveResult` |
| `plugin.deploy` | Deploy the saved plugin workspace to the local Claude Code install target | true | `id*: json` | `DeployResult` |
| `plugin.deploy.preview` | Preview changed, skipped, and removed files before deploying the workspace | false | `id*: json` | `DeployPreviewResult` |
| `artifact.fork` | Fork artifact(s) or an entire plugin into your stash with upstream tracking. | false | `plugin_id*: json`<br>`artifacts: json`<br>`instance: json` | `ForkResult` |
| `artifact.list` | List forked marketplace artifact stashes with drift status. | false | `plugin_id: json`<br>`instance: json` | `ForkedPluginStatus[]` |
| `artifact.unfork` | Remove fork tracking metadata for artifact(s) or a plugin stash. | true | `plugin_id*: json`<br>`artifacts: json`<br>`instance: json`<br>`confirm*: json` | `UnforkResult` |
| `artifact.reset` | Reset forked artifact(s) back to their upstream base snapshot. | true | `plugin_id*: json`<br>`artifacts: json`<br>`instance: json`<br>`confirm*: json` | `ResetResult` |
| `artifact.diff` | Show diffs between forked artifact content and upstream/base snapshots. | false | `plugin_id*: json`<br>`artifact_path: json`<br>`instance: json` | `ArtifactDiffResult` |
| `artifact.patch` | Apply a patch to one forked artifact in the marketplace stash. | false | `plugin_id*: json`<br>`artifact_path*: json`<br>`patch*: json`<br>`description: json`<br>`instance: json` | `PatchResult` |
| `artifact.update.check` | Check whether a forked plugin artifact stash has an upstream update | false | `plugin_id: json` | `UpdateCheckResult[]` |
| `artifact.update.preview` | Preview artifact update changes and conflicts for a forked plugin stash | false | `plugin_id*: json` | `UpdatePreviewResult` |
| `artifact.update.apply` | Apply a pending upstream artifact update to a forked plugin stash | true | `plugin_id*: json`<br>`strategy: json`<br>`confirm*: json` | `ApplyResult` |
| `artifact.merge.suggest` | Request an AI merge suggestion for one conflicted artifact file | false | `plugin_id*: json`<br>`artifact_path*: json` | `MergeSuggestResult` |
| `artifact.config.set` | Update artifact update preferences for a forked plugin stash | false | `plugin_id*: json`<br>`strategy: json`<br>`notify: json` | `ConfigSetResult` |
| `sources.add` | Register a new marketplace via `claude plugin marketplace add` | true | `repo: json`<br>`url: json`<br>`autoUpdate: json` | `AddResult` |
| `plugin.install` | Install a plugin via `claude plugin install` | true | `id*: json` | `InstallResult` |
| `plugin.uninstall` | Uninstall a plugin via `claude plugin uninstall` | true | `id*: json` | `UninstallResult` |
| `agent.list` | List ACP-compatible agents from the registry CDN | false | - | `Agent[]` |
| `agent.get` | Get details for a single ACP agent by id | false | `id*: json` | `Agent` |
| `agent.install` | Install an ACP agent on one or more devices. Writes a provider entry to `~/.lab/acp-providers.json`. Binary distributions are not yet supported in this build; npx/uvx are config-only. | true | `id*: json`<br>`node_ids*: json`<br>`platform: json`<br>`confirm*: json` | `InstallResults` |
| `plugin.cherry_pick` | Install selected components from a plugin to one or more devices | true | `plugin_id*: json`<br>`components*: json`<br>`node_ids*: json`<br>`scope*: json`<br>`project_path: json`<br>`confirm*: json` | `CherryPickResults` |
| `agent.uninstall` | Remove an installed ACP agent entry from `~/.lab/acp-providers.json` | true | `id*: json`<br>`confirm*: json` | `UninstallResult` |
| `mcp.config` | Return the resolved MCP registry base URL | false | - | `RegistryConfig` |
| `mcp.list` | List MCP servers from the registry with optional search, owner filter, and pagination. This action calls the upstream registry directly (/v1 surface). | false | `search: json`<br>`owner: json`<br>`limit: json`<br>`cursor: json`<br>`version: json`<br>`updated_since: json`<br>`featured: json`<br>`reviewed: json`<br>`recommended: json`<br>`hidden: json`<br>`tag: json` | `ServerListResponse` |
| `mcp.get` | Get details for a single MCP server by its registry name. Calls the upstream registry directly (/v1 surface). | false | `name*: json` | `ServerResponse` |
| `mcp.versions` | List available versions for a named MCP server. Calls the upstream registry directly (/v1 surface). | false | `name*: json` | `ServerListResponse` |
| `mcp.validate` | Validate a ServerJSON document against the registry schema without publishing | false | `server_json*: json` | `ValidationResult` |
| `mcp.install` | Install an MCP server from the registry to Lab gateway upstreams and/or Claude/Codex MCP clients on fleet devices. HTTP servers are added as remote URLs; stdio servers are added as command configs. Required env vars are written to ~/.lab/.env for gateway installs and embedded in the MCP client config for client installs. | true | `name*: json`<br>`gateway_ids: json`<br>`client_targets: json`<br>`bearer_token_env: json`<br>`version: json`<br>`env_values: json`<br>`confirm*: json` | `InstallResults` |
| `mcp.uninstall` | Remove a previously installed MCP server gateway upstream by gateway name | true | `gateway_name*: json`<br>`confirm*: json` | `GatewayView` |
| `mcp.meta.get` | Get Lab-owned local metadata for a stored registry server version from the local registry mirror. | false | `name*: json`<br>`version: json` | `RegistryLocalMeta` |
| `mcp.meta.set` | Set Lab-owned local metadata for a stored registry server version under `_meta["tv.tootie.lab/registry"]`. | false | `name*: json`<br>`version: json`<br>`metadata*: json`<br>`updated_by: json` | `RegistryLocalMeta` |
| `mcp.meta.delete` | Delete Lab-owned local metadata for a stored registry server version under `_meta["tv.tootie.lab/registry"]`. | false | `name*: json`<br>`version: json` | `RegistryLocalMetaDeleteResult` |
| `mcp.sync` | Trigger an immediate upstream sync of the local registry store. Rate-limited: returns rate_limited if called within 60 seconds of the last sync. | false | - | `SyncResult` |

## `acp`

Agent Client Protocol — session management and provider orchestration

Category: `ai`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `provider.list` | List available providers with health status | false | - | `Value` |
| `provider.get` | Get one provider's health and capabilities | false | `provider*: json` | `Value` |
| `provider.select` | Set the default provider for new sessions | false | `provider*: json` | `Value` |
| `session.list` | List all sessions owned by the caller | false | `principal: json` | `Value` |
| `session.get` | Get one session's summary and state | false | `session_id*: json` | `Value` |
| `session.start` | Create and start a new agent session | false | `provider: json`<br>`title: json`<br>`cwd: json`<br>`principal: json` | `Value` |
| `session.prompt` | Send a prompt to a running session | false | `session_id*: json`<br>`text*: json`<br>`principal*: json`<br>`page_context: json` | `Value` |
| `session.cancel` | Cancel a running session [destructive] | true | `session_id*: json`<br>`principal: json` | `Value` |
| `session.permission.approve` | Approve a pending provider permission request [destructive] | true | `session_id*: json`<br>`request_id*: json`<br>`option_id*: json`<br>`principal: json`<br>`confirm*: json` | `Value` |
| `session.permission.reject` | Reject a pending provider permission request | false | `session_id*: json`<br>`request_id*: json`<br>`principal: json` | `Value` |
| `session.close` | Close a session permanently [destructive] | true | `session_id*: json`<br>`principal: json` | `Value` |
| `session.events` | Get stored events for a session. ProviderInfo events of type 'tool_call_metadata' carry an optional '_meta' object relayed transparently from the originating agent; the key is absent (not null) when the agent did not inject it. ToolCallUpdate events carry merged '_meta' (outer wrapper wins over any '_meta' already present in raw_output). | false | `session_id*: json`<br>`since: json` | `{ "events": AcpEvent[], "count": number }` |
| `session.subscribe_ticket` | Issue a short-lived SSE auth ticket for browser EventSource clients | false | `session_id*: json`<br>`principal: json` | `Value` |

## `stash`

Component versioning and deployment — track, snapshot, and deploy Claude Code artefacts

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `components.list` | List all components in the stash | false | - | `ComponentSummary[]` |
| `component.get` | Get details for a single component | false | `id*: json` | `ComponentDetail` |
| `component.create` | Create a new component in the stash | false | `kind*: json`<br>`name*: json`<br>`label: json` | `ComponentDetail` |
| `component.import` | Import a local path into the stash as a new or updated component [destructive] | true | `id*: json`<br>`source_path*: json`<br>`kind: json` | `ImportResult` |
| `component.workspace` | Get the workspace (local checkout) path for a component | false | `id*: json` | `WorkspacePath` |
| `component.save` | Save (snapshot) the current workspace state for a component | false | `id*: json`<br>`label: json` | `SaveResult` |
| `component.revisions` | List saved revisions for a component | false | `id*: json` | `Revision[]` |
| `component.export` | Export a component to a local path [destructive] | true | `id*: json`<br>`output_path*: json`<br>`include_secrets: json`<br>`force: json` | `ExportResult` |
| `component.deploy` | Deploy a component to a registered target [destructive] | true | `id*: json`<br>`target_id*: json`<br>`revision_id: json` | `DeployResult` |
| `providers.list` | List registered sync providers | false | - | `Provider[]` |
| `provider.link` | Register a sync provider for a component | false | `id*: json`<br>`kind*: json`<br>`label*: json`<br>`config*: json` | `Provider` |
| `provider.push` | Push the latest component revision to a provider [destructive] | true | `id*: json`<br>`provider_id*: json` | `SyncResult` |
| `provider.pull` | Pull the latest state from a provider into the component [destructive] | true | `id*: json`<br>`provider_id*: json` | `SyncResult` |
| `targets.list` | List registered deploy targets | false | - | `Target[]` |
| `target.add` | Register a new deploy target | false | `name*: json`<br>`kind*: json`<br>`path: json`<br>`gateway_id: json` | `Target` |
| `target.remove` | Remove a registered deploy target [destructive] | true | `id*: json` | `RemoveResult` |

## `radarr`

Movie collection manager for Usenet and BitTorrent

Category: `servarr`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `system.status` | Return Radarr system status and version | false | - | `SystemStatus` |
| `system.health` | Return Radarr health check results | false | - | `HealthCheck[]` |
| `system.disk-space` | Return disk space information for all drives | false | - | `DiskSpace[]` |
| `system.logs` | Return list of available log files | false | - | `LogFile[]` |
| `system.updates` | Return available Radarr updates | false | - | `UpdateInfo[]` |
| `system.restart` | Restart the Radarr application | true | - | `void` |
| `system.backup` | List available Radarr backup files | false | - | `BackupFile[]` |
| `system.task` | List all scheduled background tasks | false | - | `Task[]` |
| `system.task-execute` | Execute a named scheduled task immediately | true | `name*: json` | `Command` |
| `movie.list` | List all movies in the Radarr library | false | - | `Movie[]` |
| `movie.get` | Get a single movie by its Radarr ID | false | `id*: json` | `Movie` |
| `movie.lookup` | Search for movies to add (TMDB / IMDB lookup) | false | `query*: json` | `MovieLookup[]` |
| `movie.add` | Add a movie to Radarr for monitoring and download | false | `tmdb_id*: json`<br>`title*: json`<br>`quality_profile_id*: json`<br>`root_folder_path*: json`<br>`monitored: json`<br>`year: json` | `Movie` |
| `movie.edit` | Update an existing movie resource (PUT full resource) | false | `id*: json`<br>`body*: json` | `Movie` |
| `movie.delete` | Delete a movie from Radarr | true | `id*: json`<br>`delete_files: json` | `void` |
| `queue.list` | List all items currently in the download queue | false | - | `QueueItem[]` |
| `queue.remove` | Remove an item from the download queue | true | `id*: json`<br>`remove_from_client: json`<br>`blocklist: json` | `void` |
| `calendar.list` | List upcoming movie releases | false | `start: json`<br>`end: json` | `CalendarEntry[]` |
| `command.refresh` | Refresh metadata for one movie or all movies | false | `movie_id: json` | `Command` |
| `command.search` | Trigger a file search for specified movies | false | `movie_ids*: json` | `Command` |
| `command.get` | Get the status of a previously issued command | false | `id*: json` | `Command` |
| `history.list` | List download history | false | `page: json`<br>`page_size: json` | `HistoryPage` |
| `history.movie` | List history records for a specific movie | false | `id*: json` | `HistoryRecord[]` |
| `history.failed-retry` | Mark a history record as failed and trigger a retry search | false | `id*: json` | `void` |
| `blocklist.list` | List blocked releases | false | - | `BlocklistItem[]` |
| `blocklist.delete` | Delete a specific blocklist entry | true | `id*: json` | `void` |
| `release.search` | Search indexers for available releases for a movie | false | `movie_id*: json` | `Release[]` |
| `release.grab` | Grab (download) a release returned from release.search | false | `release*: json` | `Release` |
| `indexer.list` | List configured indexers | false | - | `Indexer[]` |
| `indexer.test` | Test an indexer connection | false | `id*: json` | `void` |
| `quality-profile.list` | List quality profiles | false | - | `QualityProfile[]` |
| `quality-definition.list` | List quality definitions | false | - | `QualityDefinition[]` |
| `root-folder.list` | List root folders | false | - | `RootFolder[]` |
| `tag.list` | List all tags | false | - | `Tag[]` |
| `tag.detail-list` | List tags with details (linked movies, etc.) | false | - | `TagDetail[]` |
| `download-client.list` | List configured download clients | false | - | `DownloadClient[]` |
| `download-client.test` | Test a download client connection | false | `id*: json` | `void` |
| `remote-path-mapping.list` | List remote path mappings | false | - | `RemotePathMapping[]` |
| `config.host` | Get host configuration | false | - | `HostConfig` |
| `config.naming` | Get file naming configuration | false | - | `NamingConfig` |
| `config.ui` | Get UI configuration | false | - | `UiConfig` |
| `notification.list` | List configured notifications | false | - | `Notification[]` |
| `notification.test` | Test a notification connection | false | `id*: json` | `void` |
| `import-list.list` | List configured import lists | false | - | `ImportList[]` |
| `import-list.exclusion-list` | List import list exclusions | false | - | `ImportListExclusion[]` |
| `language.list` | List available languages | false | - | `Language[]` |
| `metadata.list` | List metadata providers | false | - | `Metadata[]` |
| `filesystem.list` | Browse the server filesystem | false | `path*: json` | `FilesystemListing` |
| `wanted.missing` | List monitored movies that are missing from the library | false | `page: json`<br>`page_size: json` | `WantedPage` |
| `wanted.cutoff` | List monitored movies that do not meet the quality cutoff | false | `page: json`<br>`page_size: json` | `WantedPage` |
| `customformat.list` | List all custom formats defined in Radarr | false | - | `CustomFormat[]` |

## `sonarr`

TV series management for the Servarr stack

Category: `servarr`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `series.list` | List all TV series in the Sonarr library | false | - | `Series[]` |
| `series.get` | Get a single series by Sonarr ID | false | `id*: json` | `Series` |
| `series.lookup` | Search for series to add (TVDB lookup / search term) | false | `query*: json` | `SeriesLookup[]` |
| `series.add` | Add a TV series to Sonarr for monitoring and download | false | `tvdb_id*: json`<br>`title*: json`<br>`quality_profile_id*: json`<br>`language_profile_id*: json`<br>`root_folder_path*: json`<br>`monitored: json`<br>`series_type: json` | `Series` |
| `series.delete` | Delete a series from Sonarr | true | `id*: json`<br>`delete_files: json` | `void` |
| `episode.list` | List all episodes for a series | false | `series_id*: json` | `Episode[]` |
| `episode.get` | Get a single episode by ID | false | `id*: json` | `Episode` |
| `queue.list` | List the download queue | false | `page: json`<br>`page_size: json`<br>`series_id: json` | `QueuePage` |
| `queue.delete` | Remove an item from the download queue | true | `id*: json`<br>`blocklist: json` | `void` |
| `history.list` | List download history | false | `page: json`<br>`page_size: json`<br>`series_id: json`<br>`episode_id: json` | `HistoryPage` |
| `wanted.list` | List wanted/missing episodes | false | `page: json`<br>`page_size: json` | `WantedPage` |
| `calendar.list` | List upcoming episode air dates | false | `start: json`<br>`end: json`<br>`unmonitored: json` | `Episode[]` |
| `health` | Return Sonarr health check results | false | - | `HealthCheck[]` |
| `system.status` | Return Sonarr system status and version | false | - | `SystemStatus` |
| `tag.list` | List all tags | false | - | `Tag[]` |
| `tag.create` | Create a new tag | false | `label*: json` | `Tag` |
| `tag.delete` | Delete a tag by ID | true | `id*: json` | `void` |
| `rootfolder.list` | List all root folders | false | - | `RootFolder[]` |
| `qualityprofile.list` | List all quality profiles | false | - | `QualityProfile[]` |
| `languageprofile.list` | List all language profiles | false | - | `LanguageProfile[]` |
| `series.edit` | Update an existing series with a full series resource body | false | `id*: json`<br>`body*: json` | `Series` |
| `episode.monitor` | Set monitored state for one or more episodes | false | `episode_ids*: json`<br>`monitored*: json` | `object` |
| `wanted.cutoff` | List episodes that have not met their cutoff quality | false | `page: json`<br>`page_size: json` | `WantedPage` |
| `release.search` | Search for available releases for a series or season | false | `series_id: json`<br>`season_number: json` | `Release[]` |
| `release.grab` | Grab a release by GUID and send to download client | false | `guid*: json` | `object` |
| `history.series` | List history records for a specific series | false | `series_id*: json` | `HistoryRecord[]` |
| `history.failed-retry` | Retry a failed download by history ID | false | `id*: json` | `void` |
| `blocklist.list` | List all blocklisted releases | false | - | `BlocklistPage` |
| `blocklist.delete` | Remove a release from the blocklist by ID | true | `id*: json` | `void` |
| `episodefile.delete` | Delete an episode file from disk by ID | true | `id*: json` | `void` |
| `system.restart` | Restart the Sonarr application | true | - | `void` |
| `system.backup` | List available system backup files | false | - | `Backup[]` |

## `prowlarr`

Indexer manager for the Servarr stack

Category: `indexer`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `indexer.list` | List all configured indexers | false | - | `Value` |
| `indexer.get` | Get a single indexer by ID | false | `id*: json` | `Value` |
| `indexer.delete` | Delete an indexer by ID | true | `id*: json` | `void` |
| `indexer.test` | Test a single indexer by ID (fetches the indexer then POSTs to test endpoint) | false | `id*: json` | `Value` |
| `indexer.testall` | Test all configured indexers | false | - | `Value` |
| `indexer.categories` | List all indexer categories | false | - | `Value` |
| `history.list` | Get history with optional filters | false | `page: json`<br>`page_size: json`<br>`sort_key: json`<br>`sort_dir: json`<br>`indexer_id: json` | `Value` |
| `application.list` | List configured applications (download clients connected to Prowlarr) | false | - | `Value` |
| `application.get` | Get a single application by ID | false | `id*: json` | `Value` |
| `application.delete` | Delete an application by ID | true | `id*: json` | `void` |
| `indexer.edit` | Update an existing indexer by ID (PUT /api/v1/indexer/{id}) | false | `id*: json`<br>`body*: json` | `Value` |
| `indexer.add` | Create a new indexer (POST /api/v1/indexer) | false | `body*: json` | `Value` |
| `indexer.stats` | Get indexer statistics (GET /api/v1/indexerstats) | false | - | `Value` |
| `indexer.status` | Get indexers with errors (GET /api/v1/indexerstatus) | false | - | `Value` |
| `indexer.search` | Search across indexers (GET /api/v1/search) | false | `query*: json`<br>`indexer_ids: json`<br>`categories: json`<br>`search_type: json` | `Value` |
| `indexer.grab` | Grab a release by GUID and send to download client (POST /api/v1/search) | false | `guid*: json` | `Value` |
| `history.indexer` | Get history for a specific indexer (GET /api/v1/history?indexerId={id}) | false | `id*: json` | `Value` |
| `application.add` | Create a new application (POST /api/v1/applications) | false | `body*: json` | `Value` |
| `system.restart` | Restart the Prowlarr server (POST /api/v1/system/restart) | true | - | `void` |
| `system.backup` | Get system backups (GET /api/v1/system/backup) | false | - | `Value` |
| `tag.list` | List all tags (GET /api/v1/tag) | false | - | `Value` |
| `system.status` | Get system status (version, branch, OS, etc.) | false | - | `Value` |
| `system.health` | Get system health alerts | false | - | `Value` |

## `plex`

Plex Media Server — library browsing, session management, and playlists

Category: `media`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.info` | Get Plex Media Server identity and configuration | false | - | `Value` |
| `server.capabilities` | Get server media provider capabilities | false | - | `Value` |
| `library.list` | List all library sections (Movies, TV Shows, Music, etc.) | false | - | `Value` |
| `library.get` | Get metadata for a specific library section | false | `section_id*: json` | `Value` |
| `library.scan` | Trigger a scan of a library section for new content | false | `section_id*: json` | `Value` |
| `library.refresh` | Force a metadata refresh for a library section (re-downloads all metadata) | true | `section_id*: json` | `Value` |
| `media.search` | Search for media items across libraries | false | `query*: json`<br>`limit: json`<br>`section_id: json` | `Value` |
| `media.get` | Get metadata for a specific media item by rating key | false | `rating_key*: json` | `Value` |
| `session.list` | List all active playback sessions | false | - | `Value` |
| `session.terminate` | Terminate an active playback session | true | `session_id*: json`<br>`reason: json` | `Value` |
| `playlist.list` | List all playlists | false | - | `Value` |
| `playlist.get` | Get a specific playlist by rating key | false | `playlist_id*: json` | `Value` |
| `playlist.create` | Create a new playlist | true | `title*: json`<br>`playlist_type*: json`<br>`uri: json` | `Value` |
| `playlist.delete` | Delete a playlist by rating key | true | `playlist_id*: json` | `void` |
| `library.browse` | List all content in a library section | false | `section_id*: json`<br>`type: json`<br>`sort: json` | `Value` |
| `library.empty-trash` | Permanently remove deleted items from a library section | true | `section_id*: json` | `void` |
| `metadata.delete` | Delete a metadata item by rating key | true | `rating_key*: json` | `void` |
| `metadata.edit` | Edit metadata fields for an item (pass flat JSON object of key-value pairs) | false | `rating_key*: json`<br>`fields*: json` | `Value` |
| `metadata.refresh` | Re-download metadata from agents for a specific item | false | `rating_key*: json` | `void` |
| `session.history` | Get recently played items (session history) | false | `account_id: json`<br>`limit: json` | `Value` |
| `hubs.continue-watching` | Get Continue Watching hub items | false | - | `Value` |
| `butler.list` | List all butler tasks and their current status | false | - | `Value` |
| `butler.run` | Trigger a specific butler task by name (e.g. `BackupDatabase`) | false | `task_name*: json` | `void` |
| `item.scrobble` | Mark a media item as played | false | `rating_key*: json` | `void` |
| `item.unscrobble` | Mark a media item as unplayed | false | `rating_key*: json` | `void` |
| `updater.status` | Get the current Plex Media Server update status | false | - | `Value` |
| `health` | Check server reachability and authentication | false | - | `Value` |

## `tautulli`

Plex Media Server analytics — activity, history, statistics, and user management

Category: `media`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `activity.list` | Get current Plex activity (all active streaming sessions) | false | - | `Value` |
| `activity.stream` | Get details for a single active session by session key | false | `session_key*: json` | `Value` |
| `history.list` | Get play history with optional filters | false | `page: json`<br>`page_size: json`<br>`order_dir: json`<br>`media_type: json`<br>`user_id: json`<br>`section_id: json`<br>`rating_key: json` | `Value` |
| `users.list` | List all Tautulli users | false | - | `Value` |
| `users.get` | Get user details by user ID | false | `user_id*: json` | `Value` |
| `users.watch_time` | Get watch time statistics for a user | false | `user_id*: json` | `Value` |
| `users.player_stats` | Get player statistics for a user | false | `user_id*: json` | `Value` |
| `libraries.list` | List all Plex libraries tracked by Tautulli | false | - | `Value` |
| `libraries.get` | Get library details by section ID | false | `section_id*: json` | `Value` |
| `libraries.media_info` | Get media info listing for a library | false | `section_id*: json` | `Value` |
| `stats.home` | Get home stats (most played, most active users, recently added) | false | `time_range: json`<br>`stats_count: json` | `Value` |
| `stats.plays_by_date` | Get play count statistics grouped by date | false | `time_range: json`<br>`y_axis: json` | `Value` |
| `media.recently-added` | Get recently added media items | false | `count: json`<br>`section_id: json` | `Value` |
| `media.metadata` | Get metadata for a media item by rating key | false | `rating_key*: json` | `Value` |
| `media.children` | Get children metadata for a media item by rating key | false | `rating_key*: json` | `Value` |
| `media.export-metadata` | Export metadata for a media item | false | `rating_key*: json`<br>`media_type*: json` | `Value` |
| `user.item-stats` | Get user statistics for a media item by rating key | false | `rating_key*: json`<br>`media_type: json` | `Value` |
| `user.delete-history` | Delete all play history for a user (permanent) | true | `user_id*: json` | `Value` |
| `plays.by-day` | Get play count grouped by day of week | false | `time_range: json` | `Value` |
| `plays.by-hour` | Get play count grouped by hour of day | false | `time_range: json` | `Value` |
| `plays.by-stream-type` | Get play count grouped by stream type (transcode vs direct play) | false | `time_range: json` | `Value` |
| `plays.by-month` | Get play count grouped by month | false | `time_range: json` | `Value` |
| `server.pms-update` | Check for Plex Media Server updates | false | - | `Value` |
| `system.info` | Get Tautulli server info and status | false | - | `Value` |
| `system.settings` | Get Tautulli settings | false | - | `Value` |

## `sabnzbd`

Usenet download client

Category: `download`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `server.version` | Return SABnzbd version string | false | - | `string` |
| `queue.list` | List items in the download queue with status and progress | false | - | `QueueResponse` |
| `queue.delete` | Delete an item from the download queue | true | `nzo_id*: json` | `bool` |
| `history.list` | List download history | false | `limit: json` | `HistoryResponse` |
| `history.delete` | Delete a single history item | true | `nzo_id*: json` | `bool` |
| `history.purge` | Purge all completed history items | true | - | `bool` |
| `server.stats` | Return download statistics (total/month/week/day) | false | - | `ServerStats` |
| `server.warnings` | List SABnzbd warnings | false | - | `string[]` |
| `queue.pause` | Pause the download queue | false | - | `bool` |
| `queue.resume` | Resume the download queue | false | - | `bool` |
| `queue.speed.limit` | Set the download speed limit in KB/s (0 = unlimited) | false | `kbps*: json` | `bool` |
| `queue.addurl` | Add a URL (NZB, magnet, or direct link) to the download queue | false | `url*: json`<br>`cat: json`<br>`priority: json` | `object` |
| `history.retry` | Retry a single failed history item by NZO ID | false | `nzo_id*: json` | `bool` |
| `history.retry-all` | Retry all failed history items | false | - | `bool` |
| `server.fullstatus` | Return detailed server status including connection info and queue details | false | - | `object` |
| `category.list` | List configured download categories | false | - | `object` |
| `queue.set-complete-action` | Set the action to perform when the queue completes (e.g. shutdown, hibernate, sleep) | false | `action*: json` | `bool` |
| `pp.pause` | Pause post-processing | false | - | `bool` |
| `pp.resume` | Resume post-processing | false | - | `bool` |
| `rss.fetch-now` | Trigger an immediate RSS feed fetch | false | - | `object` |
| `config.get` | Get the full SABnzbd configuration (sensitive fields redacted) | false | - | `object` |

## `qbittorrent`

BitTorrent download client

Category: `download`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Show the schema for a specific action | false | `action*: json` | `ActionSpec` |
| `transfer.info` | Get global transfer info (speeds, limits, DHT nodes, connection status) | false | - | `TransferInfo` |
| `transfer.download.limit` | Set global download speed limit in bytes/s (0 = unlimited) | false | `limit*: json` | `{}` |
| `transfer.upload.limit` | Set global upload speed limit in bytes/s (0 = unlimited) | false | `limit*: json` | `{}` |
| `torrent.list` | List torrents, optionally filtered by state or category | false | `filter: json`<br>`category: json`<br>`limit: json` | `Torrent[]` |
| `torrent.properties` | Get detailed properties of a torrent by hash | false | `hash*: json` | `TorrentProperties` |
| `torrent.trackers` | Get tracker list for a torrent by hash | false | `hash*: json` | `Tracker[]` |
| `torrent.pause` | Pause one or more torrents (pipe-separated hashes, or 'all') | false | `hashes*: json` | `{}` |
| `torrent.resume` | Resume one or more torrents (pipe-separated hashes, or 'all') | false | `hashes*: json` | `{}` |
| `torrent.delete` | Delete one or more torrents and optionally their data | true | `hashes*: json`<br>`delete_files: json` | `{}` |
| `torrent.recheck` | Force re-hash verification of one or more torrents | false | `hashes*: json` | `{}` |
| `torrent.category.set` | Set the category for one or more torrents | false | `hashes*: json`<br>`category*: json` | `{}` |
| `torrent.download.limit` | Set per-torrent download limit in bytes/s (0 = unlimited) | false | `hashes*: json`<br>`limit*: json` | `{}` |
| `torrent.upload.limit` | Set per-torrent upload limit in bytes/s (0 = unlimited) | false | `hashes*: json`<br>`limit*: json` | `{}` |
| `category.list` | List all torrent categories | false | - | `Category[]` |
| `app.version` | Get the qBittorrent application version | false | - | `string` |
| `app.preferences` | Get application preferences (save path, rate limits, slot limits) | false | - | `Preferences` |
| `log.list` | Get application log entries | false | `last_known_id: json` | `LogEntry[]` |
| `torrent.add` | Add one or more torrents by URL (magnet links or HTTP URLs, newline-separated) | false | `urls*: json`<br>`savepath: json`<br>`category: json`<br>`tags: json` | `{}` |
| `transfer.toggle-speed-limits` | Toggle alternative speed limits mode on/off | false | - | `{}` |
| `torrent.files` | Get the file list for a torrent by hash | false | `hash*: json` | `TorrentFile[]` |
| `torrent.set-file-prio` | Set the download priority for a specific file within a torrent | false | `hash*: json`<br>`id*: json`<br>`priority*: json` | `{}` |
| `torrent.set-location` | Move one or more torrents to a new save location | false | `hashes*: json`<br>`location*: json` | `{}` |
| `torrent.add-tags` | Add tags to one or more torrents | false | `hashes*: json`<br>`tags*: json` | `{}` |
| `torrent.remove-tags` | Remove tags from one or more torrents (empty tags string removes all tags) | false | `hashes*: json`<br>`tags*: json` | `{}` |
| `torrent.reannounce` | Force re-announce one or more torrents to all their trackers | false | `hashes*: json` | `{}` |
| `torrent.set-share-limits` | Set ratio and seeding time share limits for one or more torrents | false | `hashes*: json`<br>`ratio_limit*: json`<br>`seeding_time_limit*: json`<br>`inactive_seeding_time_limit*: json` | `{}` |
| `category.create` | Create a new torrent category | false | `category*: json`<br>`savepath: json` | `{}` |
| `category.edit` | Edit an existing torrent category's save path | false | `category*: json`<br>`savepath*: json` | `{}` |
| `sync.maindata` | Get sync maindata (full state or incremental delta). Callers must store and pass back the rid from the previous response for incremental delta updates. | false | `rid: json` | `SyncMaindata` |

## `tailscale`

WireGuard-based mesh VPN — list devices, manage auth keys, query DNS settings

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `device.list` | List all devices in the tailnet | false | - | `DeviceList` |
| `device.get` | Get details for a specific device by ID | false | `device_id*: json` | `Device` |
| `device.delete` | Remove a device from the tailnet | true | `device_id*: json` | `void` |
| `device.authorize` | Authorize or de-authorize a device | false | `device_id*: json`<br>`authorized*: json` | `void` |
| `key.list` | List all auth keys for the tailnet | false | - | `KeyList` |
| `key.get` | Get details for a specific auth key | false | `key_id*: json` | `AuthKey` |
| `key.delete` | Delete an auth key | true | `key_id*: json` | `void` |
| `dns.nameservers` | Get DNS nameservers configured for the tailnet | false | - | `DnsNameservers` |
| `dns.search_paths` | Get DNS search paths configured for the tailnet | false | - | `DnsSearchPaths` |
| `dns.split-get` | Get the split DNS configuration for the tailnet | false | - | `object` |
| `dns.split-set` | Replace the split DNS configuration for the tailnet | false | `config*: json` | `object` |
| `acl.get` | Get the current ACL policy file for the tailnet | false | - | `object` |
| `acl.validate` | Validate an ACL policy file without applying it | false | `policy*: json` | `object` |
| `acl.set` | Set the ACL policy file for the tailnet (validates first) | false | `policy*: json` | `object` |
| `device.routes-get` | Get advertised and accepted routes for a device | false | `device_id*: json` | `object` |
| `device.routes-set` | Set the subnet routes for a device | false | `device_id*: json`<br>`routes*: json` | `object` |
| `device.tag` | Set tags on a device (replaces existing tags) | false | `device_id*: json`<br>`tags*: json` | `void` |
| `device.expire` | Expire the key for a device, forcing re-authentication | true | `device_id*: json` | `void` |
| `user.list` | List all users in the tailnet | false | - | `object` |
| `tailnet.settings-get` | Get tailnet-level settings | false | - | `object` |
| `tailnet.settings-patch` | Patch tailnet-level settings | false | `settings*: json` | `object` |
| `key.create` | Create a new auth key for the tailnet | false | `capabilities*: json` | `object` |

## `linkding`

Self-hosted bookmark manager with tagging and search

Category: `notes`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `bookmark.list` | List bookmarks with optional search and pagination | false | `q: json`<br>`limit: json`<br>`offset: json` | `Value` |
| `bookmark.archived.list` | List archived bookmarks with optional search and pagination | false | `q: json`<br>`limit: json`<br>`offset: json` | `Value` |
| `bookmark.get` | Retrieve a single bookmark by ID | false | `id*: json` | `Value` |
| `bookmark.check` | Check whether a URL is already bookmarked | false | `url*: json` | `Value` |
| `bookmark.create` | Create a new bookmark | false | `url*: json`<br>`title: json`<br>`description: json`<br>`notes: json`<br>`is_archived: json`<br>`unread: json`<br>`shared: json`<br>`tag_names: json`<br>`payload: json` | `Value` |
| `bookmark.update` | Partially update a bookmark (PATCH — only provided fields are changed) | false | `id*: json`<br>`url: json`<br>`title: json`<br>`description: json`<br>`notes: json`<br>`unread: json`<br>`shared: json`<br>`tag_names: json`<br>`payload: json` | `Value` |
| `bookmark.archive` | Archive a bookmark | false | `id*: json` | `Value` |
| `bookmark.unarchive` | Unarchive a bookmark | false | `id*: json` | `Value` |
| `bookmark.delete` | Delete a bookmark by ID | true | `id*: json` | `void` |
| `tag.list` | List all tags | false | - | `Value` |
| `tag.get` | Retrieve a single tag by ID | false | `id*: json` | `Value` |
| `tag.create` | Create a new tag (linkding has no tag-delete API — tags are permanent) | false | `name*: json`<br>`payload: json` | `Value` |
| `user.profile` | Retrieve user profile and preferences | false | - | `Value` |
| `bundle.list` | List all bundles (saved searches) | false | - | `Value` |
| `bundle.create` | Create a new bundle (saved search) | false | `name*: json`<br>`search_query*: json`<br>`description: json` | `Bundle` |
| `bundle.update` | Partially update a bundle (PATCH — only provided fields are changed) | false | `id*: json`<br>`payload*: json` | `Bundle` |
| `bundle.delete` | Delete a bundle by ID | true | `id*: json` | `void` |
| `bookmark.assets` | List assets attached to a bookmark (snapshots, PDFs) | false | `id*: json` | `Vec<BookmarkAsset>` |
| `bookmark.assets-upload` | Upload an asset file for a bookmark via multipart/form-data | false | `id*: json`<br>`file_name*: json`<br>`file_base64*: json` | `BookmarkAsset` |

## `memos`

Lightweight self-hosted memo hub for short-form notes and journals

Category: `notes`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `memos.list` | List memos, optionally filtered by creator, tag, or paginated | false | `creator: json`<br>`tag: json`<br>`page_size: json`<br>`page_token: json` | `Value` |
| `memos.get` | Get a single memo by resource name (e.g. "memos/123") | false | `name*: json` | `Value` |
| `memos.create` | Create a new memo | false | `content*: json`<br>`visibility: json` | `Value` |
| `memos.update` | Update a memo by resource name (PATCH — only provided fields are changed) | false | `name*: json`<br>`content: json`<br>`visibility: json` | `Value` |
| `memos.delete` | Delete a memo by resource name | true | `name*: json` | `void` |
| `tags.list` | List all tags | false | - | `Value` |
| `workspace.profile` | Get workspace profile (name, version, owner) | false | - | `Value` |
| `user.me` | Get the authenticated user's profile | false | - | `Value` |
| `user.list` | List all users (admin only; returns auth_failed for non-admins) | false | - | `Value` |
| `user.stats` | Get memo statistics for a user by resource name (e.g. "users/1" or "users/me") | false | `user*: json` | `Value` |
| `webhook.list` | List webhooks for a user by resource name | false | `user*: json` | `Value` |
| `webhook.create` | Create a webhook for a user | false | `user*: json`<br>`url*: json`<br>`name*: json` | `Value` |
| `attachment.upload` | Upload a file attachment via multipart/form-data (base64-encoded bytes) | false | `filename*: json`<br>`data_base64*: json`<br>`mime_type*: json` | `Value` |
| `attachment.delete` | Delete an attachment by resource name | true | `name*: json` | `void` |
| `memo.comment-list` | List comments on a memo | false | `name*: json` | `Value` |
| `memo.comment-create` | Create a comment on a memo | false | `name*: json`<br>`content*: json` | `Value` |
| `memo.share-list` | List share links for a memo | false | `name*: json` | `Value` |
| `memo.share-create` | Create a share link for a memo | false | `name*: json` | `Value` |

## `beads`

Git/Dolt-backed issue tracker through the local bd CLI

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `contract.status` | Return Beads local CLI integration contract status | false | - | `ContractStatus` |
| `health.status` | Check bd availability and workspace status | false | - | `BeadsHealth` |
| `version.get` | Return bd version metadata | false | - | `BdJson` |
| `context.get` | Return active Beads workspace/backend context | false | - | `BdJson` |
| `status.summary` | Return Beads database summary counts | false | - | `BdJson` |
| `issue.list` | List Beads issues with optional status and limit filters | false | `status: json`<br>`limit: json` | `BdJson` |
| `issue.ready` | List ready unblocked Beads issues | false | `limit: json` | `BdJson` |
| `issue.show` | Show one Beads issue by id | false | `id*: json` | `BdJson` |
| `graph.show` | Show one Beads issue dependency graph | false | `id*: json` | `BdJson` |

## `bytestash`

Self-hosted code snippet manager

Category: `notes`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `auth.config` | Get auth provider configuration | false | - | `Value` |
| `auth.register` | Register a new user | true | `username*: json`<br>`password*: json`<br>`payload: json` | `Value` |
| `auth.login` | Log in and receive a JWT | false | `username*: json`<br>`password*: json`<br>`payload: json` | `Value` |
| `snippets.list` | List the caller's snippets | false | - | `Value` |
| `snippets.get` | Get one snippet | false | `id*: json` | `Value` |
| `snippets.create` | Create a snippet | true | `title*: json`<br>`description: json`<br>`language: json`<br>`fragments: json`<br>`categories: json`<br>`payload: json` | `Value` |
| `snippets.update` | Update a snippet | true | `id*: json`<br>`title: json`<br>`description: json`<br>`language: json`<br>`fragments: json`<br>`categories: json`<br>`payload: json` | `Value` |
| `snippets.delete` | Delete a snippet | true | `id*: json` | `void` |
| `snippets.public.list` | List public snippets | false | - | `Value` |
| `snippets.public.get` | Get one public snippet | false | `id*: json` | `Value` |
| `snippets.share.create` | Create a share link for a snippet | true | `snippetId*: json`<br>`requiresAuth: json`<br>`expiresIn: json`<br>`payload: json` | `Value` |
| `snippets.share.get` | Get a shared snippet | false | `share_id*: json` | `Value` |
| `categories.list` | List snippet metadata including all categories in use | false | - | `Value` |
| `users.list` | List users | false | - | `Value` |
| `users.toggle-active` | Toggle a user's active status (admin only) | true | `id*: json` | `Value` |
| `users.delete` | Delete a user | true | `id*: json` | `void` |

## `paperless`

Self-hosted document management system with OCR and full-text search

Category: `documents`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `documents.list` | List documents with optional filters | false | `query: json`<br>`page: json`<br>`page_size: json`<br>`ordering: json`<br>`correspondent: json`<br>`document_type: json`<br>`tags__id__all: json` | `Value` |
| `documents.get` | Get a single document by ID | false | `id*: json` | `Value` |
| `documents.metadata` | Get full metadata for a document | false | `id*: json` | `Value` |
| `documents.update` | Partially update a document (PATCH) | true | `id*: json`<br>`title: json`<br>`correspondent: json`<br>`document_type: json`<br>`tags: json`<br>`payload: json` | `Value` |
| `documents.delete` | Delete a document permanently | true | `id*: json` | `void` |
| `tags.list` | List all tags | false | - | `Value` |
| `tags.get` | Get a tag by ID | false | `id*: json` | `Value` |
| `tags.create` | Create a tag | true | `name*: json`<br>`colour: json`<br>`is_inbox_tag: json`<br>`payload: json` | `Value` |
| `tags.delete` | Delete a tag | true | `id*: json` | `void` |
| `correspondents.list` | List all correspondents | false | - | `Value` |
| `correspondents.get` | Get a correspondent by ID | false | `id*: json` | `Value` |
| `correspondents.create` | Create a correspondent | true | `name*: json`<br>`payload: json` | `Value` |
| `correspondents.delete` | Delete a correspondent | true | `id*: json` | `void` |
| `document_types.list` | List all document types | false | - | `Value` |
| `document_types.get` | Get a document type by ID | false | `id*: json` | `Value` |
| `document_types.create` | Create a document type | true | `name*: json`<br>`payload: json` | `Value` |
| `document_types.delete` | Delete a document type | true | `id*: json` | `void` |
| `statistics` | Get instance statistics (total documents, inbox count, etc.) | false | - | `Value` |
| `tasks.list` | List async task status | false | - | `Value` |
| `document.upload` | Upload a document via multipart/form-data | false | `file_base64*: json`<br>`filename*: json`<br>`title: json`<br>`correspondent: json`<br>`document_type: json`<br>`tags: json` | `Value` |
| `document.bulk-edit` | Perform a bulk operation on multiple documents (delete, set_correspondent, set_document_type, add_tag, remove_tag, etc.) | true | `documents*: json`<br>`method*: json`<br>`parameters: json` | `Value` |
| `document.download` | Download a document and return base64-encoded content | false | `id*: json`<br>`original: json` | `Value` |
| `tag.update` | Partially update a tag (PATCH) | false | `id*: json`<br>`name: json`<br>`colour: json`<br>`match: json` | `Value` |
| `saved-view.list` | List all saved views | false | - | `Value` |
| `saved-view.create` | Create a saved view | false | `payload*: json` | `Value` |
| `custom-field.list` | List all custom fields | false | - | `Value` |
| `custom-field.create` | Create a custom field | false | `name*: json`<br>`data_type*: json` | `Value` |
| `storage-path.list` | List all storage paths | false | - | `Value` |
| `storage-path.create` | Create a storage path | false | `payload*: json` | `Value` |

## `arcane`

Docker management UI — environments, containers, images, and volumes

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `health` | Check Arcane API health | false | - | `HealthResponse` |
| `environment.list` | List all registered Docker environments | false | - | `Environment[]` |
| `environment.get` | Get details for a specific environment by ID | false | `id*: json` | `Environment` |
| `container.list` | List all containers in an environment | false | `env_id*: json` | `Container[]` |
| `container.get` | Get details for a specific container | false | `env_id*: json`<br>`container_id*: json` | `Container` |
| `container.start` | Start a stopped container | false | `env_id*: json`<br>`container_id*: json` | `ContainerActionResult` |
| `container.stop` | Stop a running container | false | `env_id*: json`<br>`container_id*: json` | `ContainerActionResult` |
| `container.restart` | Restart a container | false | `env_id*: json`<br>`container_id*: json` | `ContainerActionResult` |
| `container.redeploy` | Redeploy a container (pull latest image and recreate) | true | `env_id*: json`<br>`container_id*: json` | `ContainerActionResult` |
| `project.list` | List all Compose/Docker projects in an environment | false | `env_id*: json` | `Project[]` |
| `project.create` | Create a new project in an environment | false | `env_id*: json`<br>`body*: json` | `Project` |
| `project.up` | Bring a project up (docker compose up) | false | `env_id*: json`<br>`project_id*: json` | `ProjectActionResult` |
| `project.down` | Bring a project down (docker compose down) | true | `env_id*: json`<br>`project_id*: json` | `ProjectActionResult` |
| `project.redeploy` | Redeploy a project (pull latest images and recreate) | false | `env_id*: json`<br>`project_id*: json` | `ProjectActionResult` |
| `volume.list` | List all Docker volumes in an environment | false | `env_id*: json` | `Volume[]` |
| `volume.delete` | Delete a Docker volume by name | true | `env_id*: json`<br>`volume_name*: json` | `void` |
| `volume.prune` | Prune all unused Docker volumes in an environment | true | `env_id*: json` | `PruneResult` |
| `image.list` | List all Docker images in an environment | false | `env_id*: json` | `Image[]` |
| `image.pull` | Pull a Docker image in an environment | false | `env_id*: json`<br>`image*: json` | `ImagePullResult` |
| `image.prune` | Prune unused Docker images in an environment | true | `env_id*: json` | `ImagePruneResult` |
| `image.update-summary` | Get a summary of available image updates in an environment | false | `env_id*: json` | `ImageUpdateSummary` |

## `unraid`

Unraid server GraphQL API — system info, metrics, array status, Docker, and disk management

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `system.info` | Return detailed system information (OS, CPU, versions) | false | - | `SystemInfo` |
| `system.metrics` | Return current CPU and memory utilisation metrics | false | - | `SystemMetrics` |
| `system.array` | Return array state and disk list (data, parity, cache) | false | - | `ArrayStatus` |
| `system.online` | Return whether the Unraid server reports itself as online | false | - | `object: { online: bool }` |
| `docker.list` | List all Docker containers | false | - | `Vec<DockerContainer>` |
| `docker.start` | Start a Docker container | true | `id*: json` | `void` |
| `docker.stop` | Stop a Docker container | true | `id*: json` | `void` |
| `docker.restart` | Restart a Docker container (stop then start) | true | `id*: json` | `void` |
| `disk.list` | List physical disks attached to the server | false | - | `Vec<DiskInfo>` |
| `vm.list` | List all virtual machines | false | - | `Vec<VmInfo>` |
| `vm.start` | Start a virtual machine | true | `id*: json` | `{ ok: true, id: string }` |
| `vm.stop` | Stop a virtual machine (may corrupt unsaved VM work) | true | `id*: json` | `{ ok: true, id: string }` |
| `vm.pause` | Pause a running virtual machine | true | `id*: json` | `{ ok: true, id: string }` |
| `vm.resume` | Resume a paused virtual machine | false | `id*: json` | `{ ok: true, id: string }` |
| `notification.list` | List all server notifications | false | - | `Vec<NotificationInfo>` |
| `notification.create` | Create a new server notification | false | `title*: json`<br>`description: json`<br>`importance: json`<br>`type: json` | `{ ok: true }` |
| `notification.archive` | Archive a notification by ID | true | `id*: json` | `{ ok: true, id: string }` |
| `parity.history` | Return parity check history | false | - | `Vec<ParityHistoryEntry>` |
| `parity.check-start` | Start a parity check | false | `correcting: json` | `{ ok: true }` |
| `parity.check-pause` | Pause a running parity check | false | - | `{ ok: true }` |
| `parity.check-cancel` | Cancel a running parity check (destructive — invalidates partial parity data) | true | - | `{ ok: true }` |
| `share.list` | List all user shares | false | - | `Vec<Share>` |
| `plugin.list` | List installed Unraid plugins | false | - | `Vec<Plugin>` |
| `network.list` | Return network interface information | false | - | `NetworkInfo` |
| `ups.devices` | List UPS devices attached to the server | false | - | `Vec<UpsDevice>` |
| `ups.config` | Return UPS configuration | false | - | `UpsConfig` |
| `log.read` | Read lines from a log file on the server | false | `path*: json`<br>`lines: json` | `{ content: string }` |
| `flash.status` | Return flash drive status | false | - | `FlashStatus` |
| `flash.backup` | Initiate a flash drive backup (overwrites existing backup) | true | - | `{ ok: true }` |

## `unifi`

UniFi Network Application local API

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `system.info` | Return application version and runtime metadata | false | - | `ApplicationInfo` |
| `sites.list` | List local `UniFi` sites | false | - | `Page` |
| `wans.list` | List WAN interfaces for one site | false | `site_id*: json` | `Page` |
| `vpn.site-to-site-tunnels.list` | List site-to-site VPN tunnels | false | `site_id*: json` | `Page` |
| `vpn.servers.list` | List VPN servers for one site | false | `site_id*: json` | `Page` |
| `radius.profiles.list` | List RADIUS profiles for one site | false | `site_id*: json` | `Page` |
| `device-tags.list` | List device tags for one site | false | `site_id*: json` | `Page` |
| `dpi.categories.list` | List DPI categories | false | - | `Page` |
| `dpi.applications.list` | List DPI applications | false | - | `Page` |
| `countries.list` | List supported countries | false | - | `Page` |
| `wan.get` | Inspect one WAN interface for one site | false | `site_id*: json`<br>`wan_id*: json` | `Wan` |
| `devices.list` | List adopted devices for one site | false | `site_id*: json` | `Page` |
| `devices.get` | Inspect one adopted device | false | `site_id*: json`<br>`device_id*: json` | `Device` |
| `devices.stats` | Get latest statistics for one adopted device | false | `site_id*: json`<br>`device_id*: json` | `DeviceStats` |
| `pending-devices.list` | List devices pending adoption | false | - | `Page` |
| `devices.create` | Adopt a device into a site | true | `site_id*: json`<br>`mac_address*: json`<br>`ignore_device_limit: json` | `Device` |
| `devices.port-action` | Perform an action on one port of a device | true | `site_id*: json`<br>`device_id*: json`<br>`port_idx*: json`<br>`action*: json` | `Value` |
| `devices.action` | Perform an action on a device (restart, etc.) | true | `site_id*: json`<br>`device_id*: json`<br>`action*: json` | `Value` |
| `devices.delete` | Remove (forget) an adopted device | true | `site_id*: json`<br>`device_id*: json` | `Confirmation` |
| `device.update` | Update configuration for one adopted device | true | `site_id*: json`<br>`device_id*: json` | `Device` |
| `clients.list` | List active clients for one site | false | `site_id*: json` | `Page` |
| `clients.get` | Inspect one client | false | `site_id*: json`<br>`client_id*: json` | `Client` |
| `clients.action` | Perform an action on a client (block, reconnect, etc.) | true | `site_id*: json`<br>`client_id*: json`<br>`action*: json` | `Value` |
| `client.history` | Retrieve connection history for one client by MAC address | false | `site_id*: json`<br>`client_mac*: json` | `Page` |
| `client.block` | Block a client by MAC address (cuts network access) | true | `site_id*: json`<br>`client_mac*: json` | `Confirmation` |
| `client.unblock` | Unblock a previously blocked client by MAC address | false | `site_id*: json`<br>`client_mac*: json` | `Confirmation` |
| `networks.list` | List networks for one site | false | `site_id*: json` | `Page` |
| `networks.get` | Inspect one network | false | `site_id*: json`<br>`network_id*: json` | `Network` |
| `networks.references` | List references to a network (VLANs, Wi-Fi, etc.) | false | `site_id*: json`<br>`network_id*: json` | `References` |
| `networks.create` | Create a network | true | `site_id*: json` | `Network` |
| `networks.update` | Update a network | true | `site_id*: json`<br>`network_id*: json` | `Network` |
| `networks.delete` | Delete a network | true | `site_id*: json`<br>`network_id*: json` | `Confirmation` |
| `wifi.broadcasts.list` | List Wi-Fi broadcasts (SSIDs) for one site | false | `site_id*: json` | `Page` |
| `wifi.broadcasts.get` | Inspect one Wi-Fi broadcast | false | `site_id*: json`<br>`wifi_broadcast_id*: json` | `WifiBroadcast` |
| `wifi.broadcasts.create` | Create a Wi-Fi broadcast | true | `site_id*: json` | `WifiBroadcast` |
| `wifi.broadcasts.update` | Update a Wi-Fi broadcast | true | `site_id*: json`<br>`wifi_broadcast_id*: json` | `WifiBroadcast` |
| `wifi.broadcasts.delete` | Delete a Wi-Fi broadcast | true | `site_id*: json`<br>`wifi_broadcast_id*: json` | `Confirmation` |
| `wifi.update` | Update Wi-Fi configuration for one site | true | `site_id*: json`<br>`wifi_id*: json` | `Value` |
| `hotspot.vouchers.list` | List hotspot vouchers for one site | false | `site_id*: json` | `Page` |
| `hotspot.vouchers.create` | Create a hotspot voucher | true | `site_id*: json` | `Voucher` |
| `hotspot.vouchers.delete` | Delete hotspot vouchers (with optional filter) | true | `site_id*: json`<br>`filter: json` | `Confirmation` |
| `hotspot.vouchers.get` | Inspect one hotspot voucher | false | `site_id*: json`<br>`voucher_id*: json` | `Voucher` |
| `firewall.zones.list` | List firewall zones for one site | false | `site_id*: json` | `Page` |
| `firewall.zones.get` | Inspect one firewall zone | false | `site_id*: json`<br>`firewall_zone_id*: json` | `FirewallZone` |
| `firewall.zones.create` | Create a firewall zone | true | `site_id*: json` | `FirewallZone` |
| `firewall.zones.update` | Update a firewall zone | true | `site_id*: json`<br>`firewall_zone_id*: json` | `FirewallZone` |
| `firewall.zones.delete` | Delete a firewall zone | true | `site_id*: json`<br>`firewall_zone_id*: json` | `Confirmation` |
| `firewall.policies.list` | List firewall policies for one site | false | `site_id*: json` | `Page` |
| `firewall.policies.get` | Inspect one firewall policy | false | `site_id*: json`<br>`firewall_policy_id*: json` | `FirewallPolicy` |
| `firewall.policies.create` | Create a firewall policy | true | `site_id*: json` | `FirewallPolicy` |
| `firewall.policies.update` | Replace a firewall policy | true | `site_id*: json`<br>`firewall_policy_id*: json` | `FirewallPolicy` |
| `firewall.policies.patch` | Partially update a firewall policy | true | `site_id*: json`<br>`firewall_policy_id*: json` | `FirewallPolicy` |
| `firewall.policies.ordering.get` | Get firewall policy ordering | false | `site_id*: json` | `Ordering` |
| `firewall.policies.ordering.set` | Set firewall policy ordering | true | `site_id*: json` | `Ordering` |
| `acl.rules.list` | List ACL rules for one site | false | `site_id*: json` | `Page` |
| `acl.rules.get` | Inspect one ACL rule | false | `site_id*: json`<br>`acl_rule_id*: json` | `AclRule` |
| `acl.rules.create` | Create an ACL rule | true | `site_id*: json` | `AclRule` |
| `acl.rules.update` | Update an ACL rule | true | `site_id*: json`<br>`acl_rule_id*: json` | `AclRule` |
| `acl.rules.delete` | Delete an ACL rule | true | `site_id*: json`<br>`acl_rule_id*: json` | `Confirmation` |
| `acl.rules.ordering.get` | Get ACL rule ordering | false | `site_id*: json` | `Ordering` |
| `acl.rules.ordering.set` | Set ACL rule ordering | true | `site_id*: json` | `Ordering` |
| `switching.switch-stacks.list` | List switch stacks for one site | false | `site_id*: json` | `Page` |
| `switching.switch-stacks.get` | Inspect one switch stack | false | `site_id*: json`<br>`switch_stack_id*: json` | `SwitchStack` |
| `switching.mc-lag-domains.list` | List MC-LAG domains for one site | false | `site_id*: json` | `Page` |
| `switching.mc-lag-domains.get` | Inspect one MC-LAG domain | false | `site_id*: json`<br>`mc_lag_domain_id*: json` | `McLagDomain` |
| `switching.lags.list` | List LAGs for one site | false | `site_id*: json` | `Page` |
| `switching.lags.get` | Inspect one LAG | false | `site_id*: json`<br>`lag_id*: json` | `Lag` |
| `port-profile.list` | List port profiles for one site | false | `site_id*: json` | `Page` |
| `port-profile.create` | Create a port profile for one site | true | `site_id*: json` | `PortProfile` |
| `port-profile.update` | Update an existing port profile | true | `site_id*: json`<br>`port_profile_id*: json` | `PortProfile` |
| `dns.policies.list` | List DNS policies for one site | false | `site_id*: json` | `Page` |
| `dns.policies.get` | Inspect one DNS policy | false | `site_id*: json`<br>`dns_policy_id*: json` | `DnsPolicy` |
| `dns.policies.create` | Create a DNS policy | true | `site_id*: json` | `DnsPolicy` |
| `dns.policies.update` | Update a DNS policy | true | `site_id*: json`<br>`dns_policy_id*: json` | `DnsPolicy` |
| `dns.policies.delete` | Delete a DNS policy | true | `site_id*: json`<br>`dns_policy_id*: json` | `Confirmation` |
| `traffic-matching-lists.list` | List traffic matching lists for one site | false | `site_id*: json` | `Page` |
| `traffic-matching-lists.get` | Inspect one traffic matching list | false | `site_id*: json`<br>`traffic_matching_list_id*: json` | `TrafficMatchingList` |
| `traffic-matching-lists.create` | Create a traffic matching list | true | `site_id*: json` | `TrafficMatchingList` |
| `traffic-matching-lists.update` | Update a traffic matching list | true | `site_id*: json`<br>`traffic_matching_list_id*: json` | `TrafficMatchingList` |
| `traffic-matching-lists.delete` | Delete a traffic matching list | true | `site_id*: json`<br>`traffic_matching_list_id*: json` | `Confirmation` |

## `overseerr`

Request and approval frontend for Plex, Sonarr, and Radarr

Category: `media`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `health` | Probe Overseerr reachability and auth (lightweight ping) | false | - | `HealthStatus` |
| `status` | Get current Overseerr status (version, update info) | false | - | `OverseerrStatus` |
| `request.list` | List media requests with optional filters and pagination | false | `take: json`<br>`skip: json`<br>`filter: json`<br>`sort: json`<br>`requested_by: json` | `RequestList` |
| `request.get` | Get a single request by ID | false | `id*: json` | `MediaRequest` |
| `request.create` | Create a new media request for a movie or TV show | false | `media_type*: json`<br>`media_id*: json`<br>`seasons: json`<br>`is4k: json` | `MediaRequest` |
| `request.approve` | Approve a pending media request | false | `id*: json` | `MediaRequest` |
| `request.decline` | Decline a pending media request | false | `id*: json` | `MediaRequest` |
| `request.delete` | Delete a request permanently | true | `id*: json` | `void` |
| `movie.search` | Search for movies by title or keywords | false | `query*: json`<br>`page: json` | `SearchResponse` |
| `tv.search` | Search for TV shows by title or keywords | false | `query*: json`<br>`page: json` | `SearchResponse` |
| `movie.get` | Get detailed information for a movie by TMDB ID | false | `tmdb_id*: json` | `MovieDetail` |
| `tv.get` | Get detailed information for a TV show by TMDB ID | false | `tmdb_id*: json` | `TvDetail` |
| `user.list` | List all Overseerr users with pagination | false | `take: json`<br>`skip: json` | `UserList` |
| `user.get` | Get a single user by ID | false | `id*: json` | `User` |
| `issue.list` | List issues with optional filters and pagination | false | `take: json`<br>`skip: json`<br>`filter: json`<br>`sort: json` | `IssueList` |
| `issue.get` | Get a single issue by ID | false | `id*: json` | `Issue` |
| `issue.create` | Create a new issue report for a media item | false | `issue_type*: json`<br>`message*: json`<br>`media_id*: json`<br>`problem_season: json`<br>`problem_episode: json` | `Issue` |
| `issue.comment` | Add a comment to an existing issue | false | `id*: json`<br>`message*: json` | `IssueComment` |
| `request.retry` | Retry a failed media request | false | `id*: json` | `MediaRequest` |
| `request.count` | Get request counts broken down by status | false | - | `RequestCount` |
| `issue.update` | Update the status of an issue (resolved or open) | false | `id*: json`<br>`status*: json` | `Issue` |
| `media.delete` | Delete a media item permanently by Overseerr media ID | true | `id*: json` | `void` |
| `media.update-status` | Update the status of a media item | false | `id*: json`<br>`status*: json` | `void` |
| `user.requests` | List media requests belonging to a user | false | `id*: json` | `RequestList` |
| `user.quota` | Get the quota information for a user | false | `id*: json` | `UserQuota` |
| `user.edit` | Update a user by ID with a JSON body of fields to change | false | `id*: json`<br>`body*: json` | `User` |
| `job.run` | Trigger a background job by its job ID string | false | `id*: json` | `JobResult` |
| `discover.trending` | Get trending media from the Overseerr discover endpoint | false | - | `DiscoverResponse` |

## `gotify`

Self-hosted push notification server

Category: `notifications`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `message.send` | Send a push notification to all subscribers of this app token | false | `message*: json`<br>`title: json`<br>`priority: json` | `Message` |
| `message.list` | List messages (newest first) | false | `limit: json` | `PagedMessages` |
| `message.delete` | Delete a single message by id | true | `id*: json` | `void` |
| `message.purge` | Delete ALL messages on the server | true | - | `void` |
| `app.list` | List all applications (sending channels) | false | - | `Application[]` |
| `app.create` | Create an application and return its token | false | `name*: json`<br>`description: json` | `Application` |
| `app.delete` | Delete an application and all its messages | true | `id*: json` | `void` |
| `client.list` | List all clients (receiving subscribers) | false | - | `Client[]` |
| `client.create` | Create a client and return its token | false | `name*: json` | `Client` |
| `client.delete` | Delete a client | true | `id*: json` | `void` |
| `application.update` | Update an application's name or description | false | `id*: json`<br>`name*: json`<br>`description: json` | `Application` |
| `application.messages` | List all messages for a specific application | false | `id*: json`<br>`limit: json` | `PagedMessages` |
| `application.messages-delete` | Delete ALL messages for a specific application | true | `id*: json` | `void` |
| `client.update` | Update a client's name | false | `id*: json`<br>`name*: json` | `Client` |
| `plugin.list` | List all server plugins | false | - | `Plugin[]` |
| `plugin.enable` | Enable a plugin | false | `id*: json` | `void` |
| `plugin.disable` | Disable a plugin | false | `id*: json` | `void` |
| `plugin.config-get` | Get plugin configuration as YAML text | false | `id*: json` | `{config: string}` |
| `plugin.config-set` | Set plugin configuration from a YAML string | false | `id*: json`<br>`config*: json` | `void` |
| `user.list` | List all users (admin only) | false | - | `User[]` |
| `user.create` | Create a user (admin only) | false | `name*: json`<br>`pass*: json`<br>`admin: json` | `User` |
| `user.delete` | Delete a user (admin only) | true | `id*: json` | `void` |
| `server.health` | Get server health status (no auth required) | false | - | `Health` |
| `server.version` | Get server version information | false | - | `ServerVersion` |

## `openacp`

Upstream OpenACP daemon for agent sessions and messaging adapters

Category: `ai`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `system.health` | Return OpenACP daemon health | false | - | `Value` |
| `system.version` | Return OpenACP daemon version | false | - | `Value` |
| `system.restart` | Request a daemon restart | false | - | `Value` |
| `adapters.list` | List registered channel adapters | false | - | `Value` |
| `sessions.list` | List OpenACP sessions | false | - | `Value` |
| `sessions.get` | Get one OpenACP session | false | `session_id*: json` | `Value` |
| `sessions.create` | Create an OpenACP session | false | `agent: json`<br>`workspace: json` | `Value` |
| `sessions.prompt` | Enqueue a prompt for a session | false | `session_id*: json`<br>`prompt*: json` | `Value` |
| `sessions.cancel` | Cancel an OpenACP session | false | `session_id*: json` | `void` |
| `sessions.bypass.set` | Enable or disable bypass permissions for a session | false | `session_id*: json`<br>`enabled*: json` | `Value` |
| `sessions.permission.resolve` | Resolve a pending permission request | false | `session_id*: json`<br>`permission_id*: json`<br>`option_id*: json` | `Value` |
| `sessions.archive` | Archive an OpenACP session | false | `session_id*: json` | `Value` |
| `sessions.adopt` | Adopt an existing external agent session | false | `agent*: json`<br>`agent_session_id*: json`<br>`cwd: json`<br>`channel: json` | `Value` |
| `agents.list` | List configured agents | false | - | `Value` |
| `config.get` | Return upstream-redacted OpenACP config | false | - | `Value` |
| `config.editable` | Return editable config metadata | false | - | `Value` |
| `config.patch` | Patch one safe OpenACP config value | false | `path*: json`<br>`value*: json` | `Value` |
| `topics.list` | List channel topics | false | `status: json` | `Value` |
| `topics.delete` | Delete the topic for one session | false | `session_id*: json`<br>`force: json` | `void` |
| `topics.cleanup` | Delete topics matching statuses | false | `statuses: json` | `Value` |
| `tunnel.status` | Return primary tunnel status | false | - | `Value` |
| `tunnel.list` | List active user tunnels | false | - | `Value` |
| `tunnel.create` | Create a tunnel to a local port | false | `port*: json`<br>`label: json`<br>`session_id: json` | `Value` |
| `tunnel.delete` | Stop the tunnel for one local port | false | `port*: json` | `void` |
| `tunnel.delete_all` | Stop all user tunnels | false | - | `Value` |
| `notify.send` | Send a notification to registered channel adapters | false | `message*: json` | `Value` |

## `openai`

OpenAI API (chat, embeddings, models, images, audio)

Category: `ai`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `model.list` | List available models | false | - | `Model[]` |
| `chat.complete` | Create a chat completion | false | `model*: json`<br>`messages*: json`<br>`temperature: json`<br>`max_tokens: json` | `ChatCompletionResponse` |
| `embed.create` | Create embeddings for one or more input strings | false | `model*: json`<br>`input: json`<br>`inputs: json`<br>`dimensions: json` | `EmbeddingsResponse` |
| `server.health` | Check whether the OpenAI API (or compatible server) is reachable | false | - | `void` |

## `notebooklm`

Google NotebookLM notebooks, sources, and research artifacts

Category: `ai`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `notebook.list` | List NotebookLM notebooks | false | - | `Notebook[]` |
| `notebook.create` | Create a NotebookLM notebook | false | `title*: json` | `Notebook` |
| `notebook.get` | Get NotebookLM notebook details | false | `notebook_id*: json` | `Notebook` |
| `notebook.delete` | Delete a NotebookLM notebook | true | `notebook_id*: json` | `DeleteResult` |
| `source.list` | List sources in a NotebookLM notebook | false | `notebook_id*: json` | `Source[]` |
| `source.add_url` | Add a URL source to a NotebookLM notebook | false | `notebook_id*: json`<br>`url*: json` | `Source` |
| `server.health` | Check whether NotebookLM is reachable and authenticated | false | - | `Health` |

## `qdrant`

Vector database for similarity search and RAG

Category: `ai`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.health` | Check whether the Qdrant server is healthy | false | - | `void` |
| `collections.list` | List collection names | false | - | `CollectionDescription[]` |
| `collections.get` | Get raw metadata for one collection | false | `name*: json` | `Value` |
| `collection.create` | Create a new collection with vector configuration (PUT /collections/{name}) | false | `name*: json`<br>`size*: json`<br>`distance*: json` | `Value` |
| `collection.delete` | Delete a collection and all its data (DELETE /collections/{name}) | true | `name*: json` | `void` |
| `point.upsert` | Upsert points into a collection (PUT /collections/{name}/points). Large batches auto-chunked at 500 points/chunk. Pass returned rid on subsequent calls. | false | `collection*: json`<br>`points*: json` | `Value` |
| `point.search` | Search for nearest neighbours (POST /collections/{name}/points/search) | false | `collection*: json`<br>`vector*: json`<br>`limit*: json`<br>`filter: json`<br>`with_payload: json`<br>`score_threshold: json` | `Value` |
| `point.query` | Universal query endpoint (POST /collections/{name}/points/query). Accepts any Qdrant query body. | false | `collection*: json`<br>`query*: json` | `Value` |
| `point.scroll` | Scroll through points in a collection (POST /collections/{name}/points/scroll) | false | `collection*: json`<br>`scroll: json` | `Value` |
| `point.count` | Count points matching an optional filter (POST /collections/{name}/points/count) | false | `collection*: json`<br>`filter: json` | `Value` |
| `point.delete` | Delete points by ids or filter (POST /collections/{name}/points/delete) | true | `collection*: json`<br>`points: json`<br>`filter: json` | `Value` |
| `snapshot.create` | Create a collection snapshot (POST /collections/{name}/snapshots) | false | `collection*: json` | `SnapshotInfo` |
| `index.create` | Create a payload field index (PUT /collections/{name}/index) | false | `collection*: json`<br>`field_name*: json`<br>`field_schema*: json` | `Value` |

## `tei`

Hugging Face TEI server — embeddings, rerankers, sequence classification

Category: `ai`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.health` | Check whether the TEI server is healthy | false | - | `void` |
| `server.info` | Get served model and runtime metadata | false | - | `Info` |
| `embed.create` | Generate embeddings for one or more input strings | false | `input: json`<br>`inputs: json`<br>`normalize: json`<br>`truncate: json`<br>`payload: json` | `number[][]` |
| `embed.rerank` | Rerank texts against a query (POST /rerank). Max 100 texts per call — split larger batches across multiple requests. | false | `query*: json`<br>`texts*: json`<br>`truncate: json`<br>`raw_scores: json` | `RerankHit[]` |
| `embed.tokenize` | Tokenize one or more input strings (POST /tokenize). Returns token-id sequences. | false | `inputs*: json`<br>`add_special_tokens: json` | `u32[][]` |
| `embed.similarity` | Compute pairwise similarity scores for sentence pairs (POST /similarity). Inputs must be an array of [string, string] pairs. | false | `inputs*: json` | `f32[]` |
| `embed.sparse` | Generate sparse (SPLADE-style) embeddings for one or more inputs (POST /embed_sparse). | false | `inputs*: json`<br>`truncate: json` | `SparseToken[][]` |
| `embed.openai` | Generate embeddings via the OpenAI-compatible endpoint (POST /v1/embeddings). Body and response are passed through as raw JSON. | false | `body*: json` | `json` |

## `apprise`

Universal notification dispatcher (100+ services behind one URL scheme)

Category: `notifications`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.health` | Check whether the Apprise API is healthy | false | - | `HealthStatus` |
| `notify.send` | Send a stateless notification to the supplied URLs | false | `body: json`<br>`urls: json`<br>`title: json`<br>`type: json`<br>`format: json`<br>`tag: json`<br>`payload: json` | `void` |
| `notify.key.send` | Send a notification using a stored config key | false | `key*: json`<br>`body: json`<br>`title: json`<br>`type: json`<br>`format: json`<br>`tag: json`<br>`payload: json` | `void` |
| `config.add` | Persist a YAML/text Apprise config blob under a named key | false | `key*: json`<br>`config*: json`<br>`format: json` | `void` |
| `config.get` | Retrieve the stored config blob for a named key | false | `key*: json` | `{config: string}` |
| `config.delete` | Delete the stored config for a named key | true | `key*: json` | `void` |
| `config.urls` | List the notification URLs stored under a named key | false | `key*: json` | `Vec<{url: string, tags: Vec<string>}>` |
| `server.details` | Retrieve server details listing all loaded Apprise plugins | false | - | `json` |

## `deploy`

Build-and-push the lab release binary to SSH targets with integrity verification

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | List deploy actions | false | - | `Catalog` |
| `schema` | Per-action JSON schema | false | `action*: json` | `Schema` |
| `config.list` | Lists the current deploy configuration. Note: configuration is loaded once at startup — restart the lab process to pick up changes to ~/.ssh/config or deploy preferences. | false | - | `ConfigListing` |
| `plan` | Dry-run: resolve targets, hash local artifact, show what would happen | false | `targets*: json` | `DeployPlan` |
| `run` | Build, transfer, install, restart, verify on targets (destructive) | true | `targets*: json`<br>`confirm*: json`<br>`max_parallel: json`<br>`fail_fast: json` | `DeployRunSummary` |
| `rollback` | Restore the most recent timestamped backup on the specified targets (destructive) | true | `targets*: json`<br>`confirm*: json` | `DeployRunSummary` |

## `fs`

Workspace filesystem browser (read-only, deny-listed)

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `fs.list` | List immediate entries of a directory inside the configured workspace root | false | `path: json` | `{entries: [{name, path, kind, size, modified, accessible}], truncated: bool}` |

## `dozzle`

Read-only container log observation through Dozzle

Category: `bootstrap`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.health` | Probe Dozzle health | false | - | `HealthResponse` |
| `server.version` | Fetch Dozzle version | false | - | `VersionResponse` |
| `containers.list` | Return the first bounded Dozzle container inventory event | false | `max_events: json`<br>`max_bytes: json`<br>`timeout_ms: json` | `ContainersListResponse` |
| `logs.fetch` | Fetch bounded container logs as parsed JSONL events | false | `host*: json`<br>`container_id*: json`<br>`stdout: json`<br>`stderr: json`<br>`max_lines: json`<br>`max_bytes: json`<br>`timeout_ms: json` | `LogFetchResponse` |
| `logs.fetch-plain` | Fetch bounded container logs as plain text | false | `host*: json`<br>`container_id*: json`<br>`stdout: json`<br>`stderr: json`<br>`max_lines: json`<br>`max_bytes: json`<br>`timeout_ms: json` | `PlainLogFetchResponse` |

## `immich`

Self-hosted photo and video metadata

Category: `media`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.health` | Probe Immich server reachability | false | - | `null` |
| `server.info` | Fetch allowlisted Immich server info | false | - | `ServerInfo` |
| `server.version` | Fetch Immich server version | false | - | `ServerInfo` |
| `user.me` | Fetch the authenticated Immich user summary | false | - | `UserMe` |
| `asset.search` | Search Immich assets with a hard result cap | false | `query: json`<br>`limit*: json`<br>`page: json` | `AssetSearchResponse` |
| `asset.get` | Fetch redacted metadata for one Immich asset | false | `id*: json` | `AssetMetadata` |

## `jellyfin`

Jellyfin media server inventory and operator status

Category: `media`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `system.ping` | Ping the Jellyfin server | false | - | `String` |
| `system.info` | Fetch authenticated Jellyfin system information | false | - | `SystemInfo` |
| `system.public_info` | Fetch public Jellyfin system information | false | - | `PublicSystemInfo` |
| `users.list` | List Jellyfin users | false | - | `Vec<User>` |
| `users.me` | Fetch the current Jellyfin user | false | - | `User` |
| `libraries.list` | List Jellyfin virtual folders/libraries | false | - | `Vec<VirtualFolder>` |
| `items.search` | Search Jellyfin items with a bounded read-only query subset | false | `user_id: json`<br>`search_term: json`<br>`parent_id: json`<br>`include_item_types: json`<br>`recursive: json`<br>`start_index: json`<br>`limit: json` | `ItemsResult` |
| `items.get` | Fetch one Jellyfin item by ID | false | `item_id*: json` | `Item` |
| `items.counts` | Fetch Jellyfin item counts | false | - | `ItemCounts` |
| `sessions.list` | List active Jellyfin sessions | false | - | `Vec<SessionInfo>` |
| `plugins.list` | List installed Jellyfin plugins | false | - | `Vec<PluginInfo>` |

## `navidrome`

Subsonic-compatible music library metadata

Category: `media`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.ping` | Ping Navidrome through the Subsonic API | false | - | `QueryResponse` |
| `artist.list` | List Navidrome artists | false | - | `QueryResponse` |
| `album.list` | List albums with bounded Subsonic size/offset | false | `type: json`<br>`size*: json`<br>`offset: json` | `QueryResponse` |
| `album.get` | Fetch one album by string id | false | `id*: json` | `QueryResponse` |
| `search.query` | Search artists, albums, and songs with bounded size/offset | false | `query*: json`<br>`size*: json`<br>`offset: json` | `QueryResponse` |
| `playlist.list` | List playlists | false | - | `QueryResponse` |

## `scrutiny`

SMART disk health summaries

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.health` | Probe Scrutiny health endpoint | false | - | `null` |
| `dashboard.summary` | Fetch redacted Scrutiny dashboard summary | false | - | `ScrutinyResponse` |
| `device.list` | Fetch redacted Scrutiny device list | false | - | `ScrutinyResponse` |

## `freshrss`

FreshRSS feed reader via Google Reader API

Category: `notes`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `subscription.list` | List FreshRSS subscriptions | false | - | `FreshRssResponse` |
| `tag.list` | List FreshRSS tags | false | - | `FreshRssResponse` |
| `unread.counts` | Fetch FreshRSS unread counts | false | - | `FreshRssResponse` |
| `stream.items` | Fetch one bounded FreshRSS reading-list page | false | `n*: json`<br>`continuation: json` | `FreshRssResponse` |

## `loggifly`

Docker log/event monitor local heartbeat/config status

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `contract.status` | Return LoggiFly local integration contract status | false | - | `ContractStatus` |
| `health.status` | Inspect the documented LoggiFly heartbeat file health status | false | - | `HealthStatus` |
| `config.summary` | Summarize allowlisted LoggiFly config.yaml without returning raw config | false | - | `ConfigSummary` |

## `adguard`

AdGuard Home DNS filtering and query-log summaries

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.status` | Fetch AdGuard Home status | false | - | `AdguardResponse` |
| `server.version` | Fetch AdGuard Home version | false | - | `AdguardResponse` |
| `stats.summary` | Fetch AdGuard Home stats summary | false | - | `AdguardResponse` |
| `querylog.search` | Search bounded AdGuard Home DNS query log | false | `limit*: json`<br>`search: json`<br>`older_than: json` | `QueryLogResponse` |
| `filtering.status` | Fetch DNS filtering status | false | - | `AdguardResponse` |
| `filtering.check-host` | Check whether a hostname is filtered | false | `host*: json` | `AdguardResponse` |

## `glances`

Host metrics via Glances REST API v4

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.health` | Probe Glances API health | false | - | `null` |
| `system.info` | Fetch Glances system info | false | - | `GlancesResponse` |
| `quicklook.summary` | Fetch Glances quicklook summary | false | - | `GlancesResponse` |
| `cpu.summary` | Fetch Glances CPU summary | false | - | `GlancesResponse` |
| `memory.summary` | Fetch Glances memory summary | false | - | `GlancesResponse` |
| `load.summary` | Fetch Glances load summary | false | - | `GlancesResponse` |
| `network.summary` | Fetch Glances network summary | false | - | `GlancesResponse` |
| `diskio.summary` | Fetch Glances disk I/O summary | false | - | `GlancesResponse` |
| `fs.summary` | Fetch Glances filesystem summary | false | - | `GlancesResponse` |
| `process.top` | Fetch top Glances processes with process args redacted | false | `limit*: json` | `GlancesResponse` |

## `uptime-kuma`

Self-hosted uptime monitor with a Socket.IO-backed API

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `contract.status` | Show the Uptime Kuma integration contract and live action status | false | - | `UptimeKumaResponse` |
| `server.health` | Probe the Uptime Kuma web UI root | false | - | `null` |
| `monitor.list` | List monitors through the Socket.IO actor | false | - | `UptimeKumaResponse` |
| `monitor.get` | Fetch one monitor through the Socket.IO actor | false | `id*: json` | `UptimeKumaResponse` |
| `monitor.create` | Create a monitor through the Socket.IO actor | true | `config*: json` | `UptimeKumaResponse` |
| `monitor.update` | Update a monitor through the Socket.IO actor | true | `id*: json`<br>`config*: json` | `UptimeKumaResponse` |
| `monitor.delete` | Delete a monitor through the Socket.IO actor | true | `id*: json` | `UptimeKumaResponse` |
| `monitor.pause` | Pause monitor checks through the Socket.IO actor | true | `id*: json` | `UptimeKumaResponse` |
| `monitor.resume` | Resume monitor checks through the Socket.IO actor | true | `id*: json` | `UptimeKumaResponse` |
| `heartbeat.list` | List heartbeat rows through the Socket.IO actor | false | `monitor_id*: json`<br>`hours: json` | `UptimeKumaResponse` |
| `status.summary` | Summarize monitor status through the Socket.IO actor | false | - | `UptimeKumaResponse` |
| `notification.list` | List notification channels through the Socket.IO actor | false | - | `UptimeKumaResponse` |
| `notification.test` | Send a real notification through the Socket.IO actor | true | `notification_id*: json` | `UptimeKumaResponse` |
| `notification.add` | Create a notification channel through the Socket.IO actor | true | `config*: json` | `UptimeKumaResponse` |

## `pihole`

Pi-hole v6 DNS sinkhole API

Category: `network`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `server.summary` | Fetch Pi-hole summary counters | false | - | `PiholeResponse` |
| `server.settings` | Fetch Pi-hole settings | false | - | `PiholeResponse` |
| `blocking.status` | Fetch DNS blocking status | false | - | `PiholeResponse` |
| `blocking.set` | Enable or disable DNS blocking | true | `blocking*: json`<br>`timer_seconds: json` | `PiholeResponse` |
| `querylog.search` | Search bounded Pi-hole query logs | false | `offset: json`<br>`limit*: json` | `QueryLogResponse` |
| `adlist.list` | List Pi-hole adlists | false | - | `PiholeResponse` |
| `adlist.add` | Add a Pi-hole adlist | true | `address*: json` | `PiholeResponse` |
| `adlist.remove` | Remove a Pi-hole adlist by id | true | `id*: json` | `PiholeResponse` |
| `domain.list` | List Pi-hole domain rules | false | - | `PiholeResponse` |
| `domain.add` | Add a Pi-hole domain rule | true | `domain*: json`<br>`domain_type*: json`<br>`comment: json` | `PiholeResponse` |

## `neo4j`

Neo4j graph database over the Bolt protocol

Category: `ai`  Status: `available`

| Action | Description | Destructive | Params | Returns |
|---|---|---:|---|---|
| `help` | Show this action catalog | false | - | `Catalog` |
| `schema` | Return the parameter schema for a named action | false | `action*: json` | `Schema` |
| `cypher.read` | Execute a read-only Cypher statement with parameter substitution | false | `statement*: json`<br>`parameters: json` | `CypherResponse` |
| `cypher.write` | Execute a write Cypher statement with parameter substitution | true | `statement*: json`<br>`parameters: json` | `WriteResponse` |
| `schema.labels` | List labels | false | - | `CypherResponse` |
| `schema.relationships` | List relationship types | false | - | `CypherResponse` |
| `schema.constraints` | List constraints | false | - | `CypherResponse` |
| `schema.indexes` | List indexes | false | - | `CypherResponse` |
| `db.list` | List databases | false | - | `CypherResponse` |
| `db.info` | Show database info | false | `database: json` | `CypherResponse` |
| `server.status` | Fetch Neo4j server component status | false | - | `CypherResponse` |
| `txn.run` | Run multiple parameterized statements in read or write mode | true | `statements*: json`<br>`mode*: json` | `object` |
