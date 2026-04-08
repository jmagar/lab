# Lab Docs

This directory is the documentation entrypoint for `lab`.

The docs are split by topic so contributors do not have to recover architecture, protocol rules, product behavior, and operator workflows from one large design document.

## Start Here

- Read [ARCH.md](./ARCH.md) to understand the crate split, runtime surfaces, and shared contracts.
- Read [CONVENTIONS.md](./CONVENTIONS.md) before changing implementation patterns or core APIs.
- Use [SERVICES.md](./SERVICES.md), [CLI.md](./CLI.md), [MCP.md](./MCP.md), and [TUI.md](./TUI.md) for surface-specific behavior.
- Use [CONFIG.md](./CONFIG.md), [EXTRACT.md](./EXTRACT.md), and [OPERATIONS.md](./OPERATIONS.md) for setup and operator workflows.

## Reading Paths

### If You Are Adding or Refactoring Code

1. [ARCH.md](./ARCH.md)
2. [CONVENTIONS.md](./CONVENTIONS.md)
3. [SERVICES.md](./SERVICES.md)
4. Then the surface doc you are touching:
   [CLI.md](./CLI.md), [MCP.md](./MCP.md), or [TUI.md](./TUI.md)

### If You Are Working on Product Behavior

1. [CLI.md](./CLI.md) for command behavior
2. [MCP.md](./MCP.md) for tool and envelope behavior
3. [TUI.md](./TUI.md) for plugin manager behavior
4. [CONFIG.md](./CONFIG.md) for config and env implications

### If You Are Working on a Service Integration

1. [SERVICES.md](./SERVICES.md)
2. [ARCH.md](./ARCH.md)
3. [CONVENTIONS.md](./CONVENTIONS.md)
4. [MCP.md](./MCP.md) and [CLI.md](./CLI.md) for the public surfaces

### If You Are Operating the Project

1. [CONFIG.md](./CONFIG.md)
2. [EXTRACT.md](./EXTRACT.md)
3. [OPERATIONS.md](./OPERATIONS.md)
4. [CLI.md](./CLI.md)

## Topic Map

- [ARCH.md](./ARCH.md)
  System shape, crate boundaries, shared contracts, and runtime flow.
- [TECH.md](./TECH.md)
  Stack choices, toolchain, feature posture, verification surfaces, and release tooling.
- [MCP.md](./MCP.md)
  Transport model, one-tool-per-service design, discovery, envelopes, and destructive-op elicitation.
- [SERVICES.md](./SERVICES.md)
  Service inventory, feature gates, plugin metadata, multi-instance support, and add-a-service workflow.
- [CLI.md](./CLI.md)
  Command structure, output rules, confirmation rules, install/uninstall, and operator commands.
- [TUI.md](./TUI.md)
  Plugin manager scope, interaction model, `.mcp.json` behavior, and TUI state rules.
- [CONFIG.md](./CONFIG.md)
  Env and TOML config ownership, load order, secrets handling, and instance naming.
- [CONVENTIONS.md](./CONVENTIONS.md)
  Locked engineering rules around async, HTTP, testing, docs, API surface, and privacy.
- [EXTRACT.md](./EXTRACT.md)
  The synthetic bootstrap service, URI forms, parser strategy, and `.env` merge semantics.
- [OPERATIONS.md](./OPERATIONS.md)
  Repo helpers, doctor/health workflows, CI expectations, release behavior, and update rules.

## Canonical Source Policy

These topic docs are the source of truth for the project.

When updating behavior or decisions:

- edit the topic doc that owns that concern
- do not recreate a monolithic “master design” file
- update multiple docs only when a decision genuinely crosses boundaries

## Edit Guide

Use the smallest correct doc:

- architecture or boundaries: [ARCH.md](./ARCH.md)
- implementation rules: [CONVENTIONS.md](./CONVENTIONS.md)
- service model or inventory: [SERVICES.md](./SERVICES.md)
- CLI UX or command behavior: [CLI.md](./CLI.md)
- MCP tool, discovery, or envelope behavior: [MCP.md](./MCP.md)
- TUI behavior: [TUI.md](./TUI.md)
- config, env, secrets, instance naming: [CONFIG.md](./CONFIG.md)
- extract/bootstrap flows: [EXTRACT.md](./EXTRACT.md)
- operator workflows, CI, releases: [OPERATIONS.md](./OPERATIONS.md)
- stack and toolchain choices: [TECH.md](./TECH.md)

## Common Questions

- “Where does business logic belong?”
  See [ARCH.md](./ARCH.md).
- “What is the canonical MCP response/error shape?”
  See [MCP.md](./MCP.md).
- “How do multi-instance services work?”
  See [CONFIG.md](./CONFIG.md) and [SERVICES.md](./SERVICES.md).
- “How should a new service be added?”
  See [SERVICES.md](./SERVICES.md).
- “What rules are locked and review-enforced?”
  See [CONVENTIONS.md](./CONVENTIONS.md).
- “What is the expected CI and release behavior?”
  See [OPERATIONS.md](./OPERATIONS.md) and [TECH.md](./TECH.md).
