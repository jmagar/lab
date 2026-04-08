# TUI

The TUI is a focused plugin manager, not a second full control plane.

## Scope

The TUI exists to help users:

- inspect available services
- see install state
- toggle or apply plugin selections
- manage `.mcp.json` integration
- understand required env setup

It is not intended to replicate every CLI or MCP operation.

## Implementation

- `ratatui`
- `crossterm`
- always compiled

## Primary Screen

The core experience is a service list grouped by category with install state and concise metadata.

Each row should reflect:

- service name
- short description
- install state
- availability or missing config state

An example mental model is:

- installed vs available
- category grouping
- env readiness hints

## Data Source

The TUI should consume the same `PluginMeta` records used elsewhere.

That keeps:

- categories consistent
- env requirements consistent
- display names consistent

## Interaction Model

Expected interaction is simple:

- navigate
- toggle
- apply
- quit

Complex modal workflows should be the exception, not the baseline.

The TUI should stay optimized for quick operator setup tasks rather than deep service management.

## `.mcp.json` Integration

The TUI is allowed to drive `.mcp.json` patching, but it should use the same atomic patching semantics as the CLI install/uninstall path.

That means:

- parse existing config
- compute updated service args
- back up first
- write atomically
- verify parse after write

The TUI should not invent its own patching logic. It should reuse the same semantic rules as CLI install/uninstall.

## State Policy

TUI state is ephemeral.

Rules:

- no persisted window state
- no persisted selection state
- no config file for TUI preferences

This is a hard anti-creep rule.

## Secrets and Prompts

If the TUI prompts for env values:

- secret values must be masked
- secret values must never be echoed back in the UI
- writes should go only to the designated env file flow

## Relationship to CLI

The TUI is a convenience layer for plugin and setup workflows. The CLI remains the general-purpose operator surface.
