# Lab Docs

This directory is the documentation entrypoint for `lab`.

The docs are split by topic so contributors do not have to recover architecture, protocol rules, product behavior, and operator workflows from one large design document.

## Start Here

- Read [ARCH.md](./ARCH.md) to understand the crate split, runtime surfaces, and shared contracts.
- Read [CONVENTIONS.md](./CONVENTIONS.md) before changing implementation patterns or core APIs.
- Use [SERVICES.md](./SERVICES.md), [CLI.md](./CLI.md), [MCP.md](./MCP.md), and [TUI.md](./TUI.md) for surface-specific behavior.
- Use [CONFIG.md](./CONFIG.md), [EXTRACT.md](./EXTRACT.md), and [OPERATIONS.md](./OPERATIONS.md) for setup and operator workflows.
- Use [OBSERVABILITY.md](./OBSERVABILITY.md) for the mandatory logging, correlation, redaction, and verification contract.
- Use [ERRORS.md](./ERRORS.md) for the shared error taxonomy, envelope shapes, and status mapping contract.
- Use [SERIALIZATION.md](./SERIALIZATION.md) for the shared serde, envelope, and output-boundary contract.
- Use [DISPATCH.md](./DISPATCH.md) for the shared surface-neutral dispatch-layer contract and dependency rules.
- Use [SERVICE_LAYER_MIGRATION.md](./SERVICE_LAYER_MIGRATION.md) when executing the refactor from surface-coupled dispatch to the shared `services` layer.
- Use [SERVICE_ONBOARDING.md](./SERVICE_ONBOARDING.md) when you are bringing a new service online end to end.

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
5. [OBSERVABILITY.md](./OBSERVABILITY.md) for logging, request tracing, and redaction rules
6. [ERRORS.md](./ERRORS.md) for stable kinds and structured error behavior
7. [SERIALIZATION.md](./SERIALIZATION.md) for serde and output-boundary rules
8. [DISPATCH.md](./DISPATCH.md) for layer ownership and adapter direction
9. [SERVICE_LAYER_MIGRATION.md](./SERVICE_LAYER_MIGRATION.md) for the concrete migration phases and checklists

### If You Are Working on a Service Integration

1. [SERVICES.md](./SERVICES.md)
2. [ARCH.md](./ARCH.md)
3. [CONVENTIONS.md](./CONVENTIONS.md)
4. [MCP.md](./MCP.md) and [CLI.md](./CLI.md) for the public surfaces
5. [OBSERVABILITY.md](./OBSERVABILITY.md) for instrumentation and verification requirements
6. [ERRORS.md](./ERRORS.md) and [SERIALIZATION.md](./SERIALIZATION.md) for transport and envelope consistency
7. [DISPATCH.md](./DISPATCH.md) for shared operation ownership across CLI, MCP, and API
8. [SERVICE_LAYER_MIGRATION.md](./SERVICE_LAYER_MIGRATION.md) for the refactor sequence if you are migrating existing services

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
  Service inventory, feature gates, plugin metadata, multi-instance support, coverage docs, and add-a-service workflow.
- [SERVICE_ONBOARDING.md](./SERVICE_ONBOARDING.md)
  End-to-end checklist for adding a new service, from upstream spec to verification.
- [CLI.md](./CLI.md)
  Command structure, output rules, confirmation rules, install/uninstall, and operator commands.
- [TUI.md](./TUI.md)
  Plugin manager scope, interaction model, `.mcp.json` behavior, and TUI state rules.
- [CONFIG.md](./CONFIG.md)
  Env and TOML config ownership, load order, secrets handling, and instance naming.
- [OBSERVABILITY.md](./OBSERVABILITY.md)
  Mandatory logging boundaries, required fields, correlation rules, redaction, and verification gates.
- [ERRORS.md](./ERRORS.md)
  Shared error taxonomy, stable `kind` values, MCP and HTTP error envelopes, and status mapping.
- [SERIALIZATION.md](./SERIALIZATION.md)
  Serde ownership, stable envelope shapes, CLI output boundaries, and naming rules.
- [DISPATCH.md](./DISPATCH.md)
  Surface-neutral dispatch ownership, dependency direction, operation metadata, and adapter responsibilities.
- [SERVICE_LAYER_MIGRATION.md](./SERVICE_LAYER_MIGRATION.md)
  Phase-by-phase guide and checklist for moving existing services into the shared dispatch layer.
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
- observability, request tracing, redaction: [OBSERVABILITY.md](./OBSERVABILITY.md)
- error taxonomy and envelope rules: [ERRORS.md](./ERRORS.md)
- serialization and output-shape rules: [SERIALIZATION.md](./SERIALIZATION.md)
- dispatch-layer ownership and adapter rules: [DISPATCH.md](./DISPATCH.md)
- service-layer migration execution plan: [SERVICE_LAYER_MIGRATION.md](./SERVICE_LAYER_MIGRATION.md)
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
