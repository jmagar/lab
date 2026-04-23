# Adversarial Review: In-Process Service Gateway Implementation Plan

**Date:** 2026-04-14
**Plan:** `docs/superpowers/plans/2026-04-14-virtual-service-gateway-implementation.md` (legacy filename)
**Spec:** `docs/superpowers/specs/2026-04-14-virtual-service-gateway-contract.md` (legacy filename)
**Review stance:** hostile execution review focused on hidden coupling, stale runtime state, and scope leakage

## Findings

### 1. Canonical service config can easily become split-brain

The plan correctly says service credentials are canonical and must not be duplicated into in-process-service state, but the implementation risk is high because current Lab services are mostly env-driven and do not already expose a first-class mutable service-config store through the gateway domain.

Failure mode:

- the UI writes `.env`
- in-process-service state is written elsewhere
- the runtime reads one source for list/detail state and another source for actual client construction
- the operator sees a configured service that does not actually match the live client

Mitigation added to the plan:

- do not ship `.env` write support without live client refresh
- keep service credentials canonical in existing service config paths only

Residual requirement:

- the implementation must explicitly choose whether service-edit reads come from `.env`-derived state, TOML-backed state, or a unified abstraction
- this must be documented in the owning config docs

### 2. Visible configured-but-disabled rows need clear state semantics

The clarified product requirement is better than the earlier hidden-row model: configured services remain visible in the server list even before enablement. That removes the biggest discoverability trap, but it creates a different risk if the state model is not explicit enough.

Failure mode:

- user saves Plex credentials
- Plex appears in the server list
- the UI does not clearly distinguish `configured`, `enabled`, `disabled`, `connected`, and `disconnected`
- operators misread a greyed-out configured row as broken instead of intentionally inactive

Mitigation added to the plan:

- keep configured-but-disabled rows visible
- add list filters for `active`, `configured`, `enabled`, `disabled`, `connected`, and `disconnected`

Residual requirement:

- the list and detail views must use the same state vocabulary
- the add dialog should still detect already-configured services and offer edit/re-enable affordances

### 3. Surface toggles are under-specified unless discovery semantics are explicit

The plan says `CLI`, `API`, and `MCP` must be enforceable, but the subtle problem is discovery. If a service remains visible in a surface’s catalog/help output after being disabled on that surface, operators and agents will still think it is callable.

Failure mode:

- `MCP` disabled but tool still shows in catalog
- `CLI` disabled but help/docs still advertise commands as ready
- `API` disabled but routes remain discoverable as ordinary service endpoints

Mitigation added to the plan:

- explicitly decide and document per-surface discovery semantics before implementation

Required implementation bar:

- disabled `MCP` should not look normally available in MCP discovery
- disabled `CLI` and `API` should not only fail at execution time; their discovery/help should align with effective availability

### 4. Action-level filtering depends on real action metadata, not just service names

The contract requires filtering within single-tool services, but the plan could still be implemented poorly if the UI hardcodes action names or if the backend only enforces policies on call paths without a trustworthy source for discovery/editing.

Failure mode:

- frontend ships hardcoded action lists
- service catalog drifts
- help/schema/action-policy editor disagree

Mitigation added to the plan:

- build the Lab Gateway picker and action-policy editor from registry/action metadata rather than hand-maintained frontend constants

Residual requirement:

- services that currently generate actions dynamically must remain consumable by the action-policy editor without special-case drift

### 5. Scope creep risk remains real on the frontend

The plan is mostly disciplined, but the UI work can easily drift into “mini dashboards” because service-backed pages invite deeper product work.

Failure mode:

- page gets extra service widgets during implementation
- control-plane parity work turns into service UX work
- backend contract gets muddied by domain-specific data fetching

Mitigation:

- keep the detail page a control-plane page only
- reject any addition that fetches business/domain data from Plex, Overseerr, etc. for this phase

## Overall Assessment

The rewritten plan is materially better than the first draft because it now:

- decomposes the work into smaller executable tasks
- forces TDD checkpoints on the dangerous state transitions
- names the exact files that own each concern
- makes live client refresh and runtime policy enforcement explicit work items instead of implied follow-ups

It is viable, but only if implementation treats these as hard constraints:

- one canonical source of truth for service credentials
- live client refresh after config changes
- explicit state model for configured-but-disabled visible rows
- discovery semantics aligned with surface enablement
- metadata-driven action-policy editing

Without those constraints, the most likely failure is a UI that appears to manage in-process services while the actual runtime behavior stays inconsistent, stale, or visually ambiguous.

## Recommendation

Proceed with the plan, but treat Tasks 2 through 4 as the critical path. The frontend should not advance beyond mocked control-plane UI until:

1. canonical config writes are live
2. service clients refresh without restart
3. disabled surfaces are enforced and no longer misleadingly discoverable
4. action-level policy is backed by real registry metadata
