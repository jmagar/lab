# Lab Docs

This directory is split by topic so contributors do not have to mine a single monolithic design file for every decision.

## Reading Order

1. [ARCH.md](./ARCH.md) for system structure and boundaries
2. [TECH.md](./TECH.md) for the stack, toolchain, and build/test/release choices
3. [CONFIG.md](./CONFIG.md) for config loading, env vars, and multi-instance rules
4. [SERVICES.md](./SERVICES.md) for the service model and inventory
5. [CLI.md](./CLI.md) for command-line behavior
6. [MCP.md](./MCP.md) for the MCP server model and protocol shape
7. [TUI.md](./TUI.md) for the plugin manager UX
8. [CONVENTIONS.md](./CONVENTIONS.md) for locked engineering rules
9. [EXTRACT.md](./EXTRACT.md) for the synthetic bootstrap service
10. [OPERATIONS.md](./OPERATIONS.md) for operator workflows, CI, and release

## Canonical Docs

- [ARCH.md](./ARCH.md), [TECH.md](./TECH.md), [MCP.md](./MCP.md), [SERVICES.md](./SERVICES.md), [CLI.md](./CLI.md), [TUI.md](./TUI.md), [CONFIG.md](./CONFIG.md), [CONVENTIONS.md](./CONVENTIONS.md), [EXTRACT.md](./EXTRACT.md), and [OPERATIONS.md](./OPERATIONS.md) are the active topic docs.
- [DESIGN.md](./DESIGN.md) is the archival long-form design record and rationale dump. It remains useful, but it is no longer the only place a contributor should have to read.

## Scope

- `ARCH.md`: architecture, crate boundaries, shared flow
- `TECH.md`: stack, dependencies, tooling, platforms
- `MCP.md`: transport, discovery, envelopes, structured errors, destructive-op elicitation
- `SERVICES.md`: service inventory, feature gates, plugin metadata, adding a service
- `CLI.md`: command model, output, confirmations, operator commands
- `TUI.md`: plugin manager scope, state, `.mcp.json` integration
- `CONFIG.md`: env and TOML config, load order, secrets, multi-instance naming
- `CONVENTIONS.md`: locked implementation rules and review-enforced conventions
- `EXTRACT.md`: bootstrap scanning and `.env` write semantics
- `OPERATIONS.md`: health tooling, CI, releases, update workflow
