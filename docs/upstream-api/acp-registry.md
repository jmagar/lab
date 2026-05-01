# ACP Registry Upstream Contract

**Last updated:** 2026-05-01
**Status:** SDK metadata exists; runtime service not yet exposed

The `acp_registry` module represents an ACP registry-compatible upstream. The
current repository state only defines the feature gate and service metadata; no
runtime dispatch surface is wired.

## Source

The configured upstream base URL is read from:

```env
ACP_REGISTRY_URL=https://example.invalid
```

## Current Lab Contract

- `lab-apis` feature: `acp_registry`
- required env: none
- optional env: `ACP_REGISTRY_URL`
- CLI/MCP/API exposure: none

Before adding a public surface, create a concrete endpoint inventory here and
mirror it in [../coverage/acp_registry.md](../coverage/acp_registry.md).

