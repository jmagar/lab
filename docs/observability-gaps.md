# Observability Gap Report

**Project:** lab  
**Date:** 2026-04-29  
**Scope:** Full codebase audit — logging, metrics, and audit trail coverage  
**Summary:** ~270 gaps identified across 50+ files. 20 Critical, 10 High (detailed below), ~240 Medium/Low.

---

## Critical Gaps (20)

These gaps represent security audit trail holes, silent data loss, or complete operational blindness in core subsystems.

### 1. `src/node/enrollment/store.rs` — Enrollment lifecycle
**Functions:** `validate()`, `approve()`, `deny()`, `record_pending()`  
**Gap:** Zero logs on security-critical enrollment lifecycle. Approve/deny/validate actions and token mismatch events leave no audit trail.  
**Add:** `info!` on approve/deny with node_id + actor; `warn!` on token mismatch with expected vs actual hash; `debug!` on validate entry/exit.

### 2. `src/mcp/elicitation.rs` — Elicitation outcomes
**Functions:** `elicit_confirm()`  
**Gap:** Outcomes (Confirmed/Declined/Cancelled/Failed) never logged. A user approving a destructive action leaves zero audit trail.  
**Add:** `info!` on every outcome variant with session_id, prompt summary, and outcome; `warn!` on Failed with error context.

### 3. `src/node/ws_client.rs` — WebSocket session lifecycle
**Functions:** `connect_and_run_session()`  
**Gap:** WS connect success, session init, close frames, and reader errors all silent.  
**Add:** `info!` on connect success with remote_addr; `info!` on session init with session_id; `debug!` on close frame; `warn!` on reader errors.

### 4. `src/node/ws_client.rs` — Pending inflight saturation
**Functions:** `send_and_await()`  
**Gap:** MAX_PENDING_INFLIGHT (256) saturation is silent. `spawn_blocking` panic on sysmetrics silently zeroes the value.  
**Add:** `warn!` at 80% capacity; `error!` on saturation; `warn!` on sysmetrics panic.

### 5. `src/node/runtime.rs` — WS flush loop
**Functions:** `spawn_ws_flush_loop()`  
**Gap:** No startup log, handle dropped silently, task exit undetected.  
**Add:** `info!` on spawn; `error!` on handle drop; `error!` on unexpected task exit.

### 6. `src/dispatch/logs/ingest.rs` — Log event drops
**Functions:** `try_submit()`, `spawn_writer()`  
**Gap:** Log events dropped silently on `TrySendError::Full`. Writer task death unlogged.  
**Add:** `warn!` on every drop with channel depth; `error!` on writer task exit.

### 7. `src/dispatch/node/send.rs` — RPC cap hits
**Functions:** `send_rpc_to_node()`  
**Gap:** MAX_PENDING_RPC (1024 global) and per-node cap (32) hits produce no log.  
**Add:** `warn!` on per-node cap hit with node_id; `error!` on global cap hit; `counter!` metrics.

### 8. `src/dispatch/deploy/lock.rs` — Deploy lock contention
**Functions:** `acquire()`  
**Gap:** Lock contention timeout returns `Conflict` with zero log.  
**Add:** `warn!` on contention with lock holder identity and wait duration; `info!` on acquire success.

### 9. `src/dispatch/deploy/build.rs` — Cargo build lifecycle
**Functions:** `build_release()`  
**Gap:** Entire cargo build lifecycle silent — no start, success, artifact path, or failure details.  
**Add:** `info!` on build start; `info!` on success with artifact path + elapsed; `error!` on failure with stderr excerpt.

### 10. `src/dispatch/marketplace/mcp_dispatch.rs` — MCP install dispatch
**Functions:** `dispatch_mcp_install()`  
**Gap:** Zero metrics and logs at dispatch level.  
**Add:** `info!` on dispatch with package_id + version; `error!` on failure; `histogram!` for duration.

### 11. `src/node/log_store.rs` — Log store ingest + open
**Functions:** `ingest()`, `open()`  
**Gap:** Blocking send under burst is silent. SQLite open/migration failures are untraced.  
**Add:** `warn!` on blocking send; `info!` on open; `error!` on migration failure.

### 12. `src/mcp/services/nodes.rs` — Node enrollment security ops
**Functions:** `dispatch()` for `enrollments.approve` / `enrollments.deny`  
**Gap:** Security operations with zero audit trail.  
**Add:** `info!` audit log on approve/deny with actor, node_id, and request metadata.

### 13. `src/dispatch/logs/store.rs` — SQLite log store init
**Functions:** `open()`  
**Gap:** SQLite open, WAL init, and schema migration all silent.  
**Add:** `info!` on open; `debug!` on WAL init; `error!` on migration failure.

### 14. `src/oauth/upstream/manager.rs` — Token refresh trigger
**Functions:** `build_auth_client()`  
**Gap:** Token refresh trigger and outcome invisible.  
**Add:** `debug!` on refresh trigger; `info!` on success; `error!` on failure with upstream_id.

### 15. `src/cli/serve.rs` — Port reclaim retry loop
**Functions:** `bind_or_reclaim()`  
**Gap:** 5-retry port reclaim loop with no per-attempt log.  
**Add:** `warn!` on each retry with attempt + port + error; `info!` on bind success.

### 16. `src/cli/serve.rs` — stdio MCP server
**Functions:** `run_stdio()`  
**Gap:** stdio MCP server failure and session termination silent.  
**Add:** `info!` on session start/end; `error!` on server failure.

### 17. `src/mcp/peers.rs` — Peer notifier
**Functions:** `PeerNotifier::run()`  
**Gap:** Sender drop exits silently, stops all MCP client broadcasts.  
**Add:** `error!` on sender drop with peer count; `info!` on clean shutdown.

### 18. `src/dispatch/upstream/pool.rs` — Upstream reconnect
**Functions:** `reprobe_upstream()`  
**Gap:** Reconnect success after heartbeat failure not logged. stdio child stderr to `/dev/null`.  
**Add:** `info!` on reconnect success with downtime duration; `warn!` on stderr discarded.

### 19. `src/node/update.rs` — Remote target update stages
**Functions:** `run_remote_target()`  
**Gap:** All 6 stages (preflight, transfer, normalize, restart, verify, controller_verify) have no timing logs.  
**Add:** `info!` on each stage entry + exit with elapsed; `error!` on failure with stage name + node_id.

### 20. `src/dispatch/deploy/authz.rs` — Deploy token auth
**Functions:** `require_deploy_token()`  
**Gap:** Auth rejections leave no audit trail.  
**Add:** `warn!` audit log on rejection with caller identity, token hash prefix, and reason.

---

## High Gaps (10)

### 1. `src/dispatch/deploy/runner.rs` — Host pipeline + rollback
**Functions:** `run_host_pipeline()`, `rollback_one_host()`  
**Add:** `info!` on pipeline entry/exit per host with elapsed; `warn!` on rollback trigger.

### 2. `src/dispatch/upstream/pool.rs` — OAuth upstream connections
**Functions:** `subject_scoped_*` fns  
**Add:** `warn!` on skip with upstream_id + reason; partial tool list warning to caller.

### 3. `src/dispatch/gateway/manager.rs` — Tool search index rebuild
**Functions:** `schedule_tool_search_rebuilds()`  
**Add:** `error!` on join error; `info!` on rebuild complete with upstream_count + elapsed.

### 4. `src/node/store.rs` — Node connect/disconnect
**Functions:** `record_hello()`, `set_connected()`  
**Add:** `info!` on connect/disconnect with node_id + version + remote_addr.

### 5. `src/dispatch/logs/forward.rs` — Log forwarding errors
**Add:** `warn!` on transient failure with retry count; `error!` on retry exhaustion.

### 6. `src/node/identity.rs` — Identity resolution
**Add:** `warn!` on fetch failure; `debug!` on cache miss.

### 7. `src/oauth/upstream/refresh.rs` — Token refresh failures
**Add:** `warn!` on approaching expiry (< 5 min); `error!` on refresh failure.

### 8. `src/dispatch/marketplace/sync.rs` — Marketplace sync
**Add:** `info!` on sync start/complete with package count + elapsed; `error!` on failure.

### 9. `src/mcp/server.rs` — MCP server errors
**Add:** `warn!` on protocol error; `info!` on clean disconnect.

### 10. `src/node/sysmetrics.rs` — Sysmetrics collection
**Add:** `warn!` on collection failure (rate-limited); return zeroes only after logging.

---

## Medium / Low Gaps (~240)

Additional gaps across:
- `src/dispatch/marketplace/acp_dispatch.rs`
- `src/dispatch/upstream/pool.rs` (pool size changes, upstream removal)
- `src/dispatch/gateway/manager.rs` (config reload, projection updates)
- `src/node/runtime.rs` (task spawn/exit tracking)
- `src/dispatch/deploy/stages.rs` (stage transitions)
- `src/mcp/services/*.rs` (service dispatch audit trails)
- `src/oauth/upstream/*.rs` (OAuth flow steps)
- `apps/gateway-admin/**` (frontend API error surfacing)

---

## Work Tracking

Beads created for all 10 work groups — see `bd list --label observability` for status.
