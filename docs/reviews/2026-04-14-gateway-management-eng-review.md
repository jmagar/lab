# Engineering Review: Gateway Management Surface Implementation Plan

**Date:** 2026-04-14
**Plan:** `.claude/worktrees/agent-accba6a1/docs/superpowers/plans/2026-04-14-gateway-management.md`
**Reviewers:** architecture-strategist, code-simplicity-reviewer, security-sentinel, performance-oracle (all haiku)
**Prior review cycles:** None detected in recent commits

---

## Architecture

**Strengths:**
- Correctly places gateway management inside `dispatch/gateway/` following established patterns
- Config-as-source-of-truth with immediate reconciliation prevents state divergence
- Proper secret handling via env-var references only (`bearer_token_env`)
- Non-destructive classification is pragmatic for administrative operations
- Catalog notification logic (emit only on effective changes) is efficient
- Reuses existing `action + params` dispatch shape for consistency

**Concerns:**
- **HIGH: Runtime state synchronization race** — Old `Arc<UpstreamPool>` references held by in-flight requests become stale after pool swap. Removed gateways may still have active connections. Need explicit contract: mutations take effect for new connections only; old connections complete on their original pool.
- **HIGH: Reconcile lock deadlock risk** — Lock held during config write (I/O-bound) + pool rebuild (CPU-bound). Should pre-compute new pool outside lock, then swap under lock.
- **MEDIUM: Missing multi-config clarity** — Plan doesn't specify which config file is mutated when multiple exist in the search path (CWD vs `~/.config/lab/`). Could lead to operator confusion.
- **MEDIUM: Bearer token rotation requires manual reload** — If env vars change, pool still holds stale tokens until `gateway.reload` is called. Should document this or implement lazy token resolution.

**Suggestions:**
- Document the pool-swap contract explicitly
- Minimize critical section: pre-compute pool, then lock-swap-unlock
- Clarify config mutation target in docs

---

## Simplicity

**Over-engineering risks:**
- **11 actions is too granular.** `gateway.discovered_tools`, `gateway.discovered_resources`, `gateway.discovered_prompts` are three actions that should be optional fields on `gateway.get` or collapsed into one. Could collapse to 5-6 actions with same coverage.
- **Catalog diff + notification machinery is over-built for a homelab tool.** Gateways are rarely modified at runtime. A simpler approach: always emit `list_changed` after any mutation (skip the diff), or skip notifications entirely and let callers poll.
- **Separate `GatewayManager` class owns too much coupling.** Config path, LabConfig, runtime pool handle, notification sink, reconcile lock — this is a monolithic orchestrator. Inline the orchestration into dispatch action handlers with a few helper functions instead.
- **12 new files for state mutation.** Standard dispatch template is 4 files. `config.rs` and `manager.rs` are justified — the manager owns shared mutable state (reconcile lock, pool handle, notification sink, config path) that must be coordinated atomically across mutations. Without a central owner, that coordination logic would be duplicated across every action handler.
- **Separate `reload` action is redundant** if mutations always auto-reconcile. Keep it as CLI-only escape hatch at most.

**Estimated LOC reduction: ~20% (150-250 lines) by collapsing actions.**

**Suggestions:**
- Collapse to 6 actions: `list`, `get`, `add`, `update`, `remove`, `reload`
- Keep `manager.rs` focused on the lock/mutate/reconcile/notify cycle — don't let it become a god object
- Skip catalog diff; always emit `list_changed` after mutations
- Defer `gateway.test` with proposed configs (test by name only for MVP)

---

## Security

### Critical (3)

1. **Command injection via `gateway.add` with `command=`** — If `command` param is passed to `std::process::Command` without sanitization, arbitrary code execution is possible. **Recommendation:** Reject `command` in `gateway.add` entirely. Only allow `url`-based upstreams via API/MCP. Stdio upstreams should be static config.toml only.

2. **Config TOCTOU race** — Gap between config read and atomic rename allows concurrent writes to clobber each other. Temp file in world-readable dir enables symlink attacks. **Recommendation:** File locking around read-write cycle; temp file in same directory with `0o600` permissions.

3. **Secret exfiltration via env var names** — `gateway.list`/`gateway.get` responses include `bearer_token_env` field, leaking env var names to API callers. **Recommendation:** Replace with boolean `has_bearer_token` flag; never expose env var names in responses or logs.

### High (4)

4. **SSRF via URL validation gap** — `gateway.add` accepts arbitrary URLs. No scheme/IP range validation. Could reach cloud metadata endpoints, internal services. **Recommendation:** Whitelist `http`/`https` schemes; block loopback and RFC 1918 ranges.

5. **Misclassification as non-destructive** — `gateway.add` expands network surface (new connections), `gateway.remove` breaks dependent clients. These should be `destructive: true` with confirmation/elicitation. **Recommendation:** Mark add/update/remove as destructive.

6. **Arbitrary process execution via stdio** — Even without shell injection, unrestricted `command` allows spawning any executable. **Recommendation:** Whitelist binary names or reject stdio for dynamic gateways.

7. **Env var name injection** — Attacker could set `bearer_token_env: "AWS_ACCESS_KEY_ID"` to reference arbitrary env vars. **Recommendation:** Whitelist env var names to `LAB_*` pattern.

### Medium (3)

8. **Path traversal risk** if config path is ever user-controllable
9. **No rate limiting** on mutation actions
10. **No upstream dependency tracking** before removal

---

## Performance

### Critical Bottlenecks

1. **Full UpstreamPool rebuild on every mutation** — 20 gateways = 20 new connections created + 20 dropped per single add/remove. Estimated 1-4s per reconciliation. At 100 gateways: 5-20s. **Fix:** Delta reconciliation — only rebuild changed connections.

2. **Global reconcile lock serializes all mutations** — Second mutation waits for first to complete full reconcile. 5 concurrent adds on 50 gateways = 7.5s total. **Fix:** Reduce lock scope to just the Arc swap; pre-compute pool outside lock.

3. **Catalog snapshot diffing scales O(n*m)** — 20 gateways x 500 tools = 10,000 entries to diff per reconcile. **Fix:** Hash-based change detection or skip diffing entirely.

4. **Blocking MCP notifications during reconciliation** — Notifications wait for lock release; slow upstreams block all notifications. **Fix:** Decouple — snapshot under lock, notify outside lock.

5. **No connection timeouts specified** — One unreachable upstream blocks entire reconcile for default 30s. **Fix:** 5s connect timeout; fail fast for unreachable upstreams.

### Projected Reconciliation Latency

| Gateways | Current Plan | With Delta Reconcile |
|----------|-------------|---------------------|
| 10 | 300ms | 50ms |
| 50 | 1.5s | 100ms |
| 100 | 3s | 200ms |

---

## Failure Modes

```
CODEPATH                    | FAILURE MODE                        | RESCUED? | TEST? | USER SEES?     | LOGGED?
----------------------------|-------------------------------------|----------|-------|----------------|--------
gateway.add (command=)      | Command injection / arbitrary exec  | N        | N     | Silent         | N       **CRITICAL GAP**
gateway.add (url=)          | SSRF to internal services           | N        | N     | Silent         | N       **CRITICAL GAP**
config write                | TOCTOU race / data loss             | N        | N     | Silent         | N       **CRITICAL GAP**
gateway.list/get response   | Secret env var names exposed        | N        | N     | Silent         | N       **CRITICAL GAP**
gateway.remove              | Active connections to removed gw    | N        | N     | Silent         | N       **CRITICAL GAP**
gateway.reload              | Slow upstream blocks 30s            | N        | N     | Timeout        | N       **CRITICAL GAP**
pool rebuild                | Connection leak on partial failure  | N        | N     | Silent         | N
concurrent mutations        | Second writer clobbers first        | N        | Y     | Silent         | N
env var missing             | Bearer token not set at runtime     | ?        | N     | Auth failure   | ?
notification fanout         | Slow client blocks notification     | N        | N     | Stale catalog  | N
```

**6 CRITICAL GAPS identified** (RESCUED=N AND TEST=N AND USER SEES=Silent).

---

## NOT in Scope

Work flagged as deferrable without blocking the core objective:

| Item | Rationale |
|------|-----------|
| `gateway.test` with proposed configs | Test by name is sufficient for MVP; proposed-config testing adds a second request shape for unclear use case |
| `gateway.discovered_tools/resources/prompts` as separate actions | Merge into `gateway.get` with optional `include_discovered` param, or defer entirely |
| Catalog diff notification machinery | Always emit `list_changed` after mutations; skip the diff — simpler, same result for callers |
| ~~`GatewayManager` as separate class~~ | **STRUCK** — Manager is the correct owner of shared mutable state (lock, pool handle, config, notification sink). Inlining would duplicate coordination logic across action handlers. |
| Multi-config file mutation support | Always mutate the first found config; document this; add `--config-path` later if needed |
| Lazy token resolution | Document that `gateway.reload` is needed after env var changes; lazy lookup adds complexity |
| Rate limiting on gateway mutations | Add post-MVP when exposed to untrusted networks |
| Upstream dependency tracking | Track which clients depend on which gateways; add before production use |

---

## Summary

- **Critical issues:** 9 — Must fix before implementing (3 security critical, 4 security high, 2 performance critical)
- **Important suggestions:** 7 — Should consider (simplification, deferred work, lock scope, timeouts)
- **Minor improvements:** 5 — Nice to have (lazy pooling, async writes, rate limiting)

---

## Recommended Changes

1. **Reject `command` parameter in `gateway.add`/`gateway.update`** — Stdio upstreams must be static config.toml only. Eliminates command injection and arbitrary process execution vectors.
2. **Mark `gateway.add/update/remove` as `destructive: true`** — These expand/contract network surface and modify persistent state. Require confirmation via elicitation (MCP) and `--yes` (CLI).
3. **Implement delta reconciliation** — Only rebuild connections for changed gateways. Pre-compute new pool outside lock, swap under lock. Add 5s connect timeout.
4. **Redact `bearer_token_env` from all responses** — Replace with `has_bearer_token: bool`. Whitelist env var names to `LAB_*` pattern.
5. **Collapse to 6 actions** — `list`, `get`, `add`, `update`, `remove`, `reload`. Merge discovered capabilities into `get`. Defer proposed-config testing.
6. **Keep `GatewayManager` focused** — It correctly owns the lock/mutate/reconcile/notify cycle. Ensure it doesn't grow into a god object; param parsing, catalog constants, and response types stay in their own files.
7. **Add URL validation** — Whitelist `http`/`https` schemes; block loopback and private IP ranges.
8. **Use file locking for config writes** — Exclusive lock around read-modify-write cycle; temp file in same directory with restrictive permissions.

---

## Completion Summary

```
Architecture issues: 4  |  Simplicity: 5  |  Security: 10  |  Performance: 5
Critical gaps: 6  |  TODOs proposed: 8
```
