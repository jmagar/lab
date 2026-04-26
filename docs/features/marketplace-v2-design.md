# Marketplace V2 Design Spec

**Status:** Approved for `/dev/marketplace` implementation  
**Route:** `/dev/marketplace` preview, later `/marketplace` production route  
**Primary references:** [Component Development Process](../design/component-development.md), [Labby Design System Contract](../design/design-system-contract.md), Gateways page patterns  

## Goal

Marketplace v2 turns the Marketplace page into a dense operator catalog for three inventory types:

- Lab plugins
- MCP Registry servers
- ACP agents

The page should stop treating these as tabbed sub-pages. It should behave like the Gateways page: one searchable catalog, one filter rail, clear summary lenses, and a density switch between visual cards and a condensed table/list view.

## Current Problems

- The current page has two competing tab systems: `Browse / Installed / Marketplaces` and `All / Plugins / MCP Servers / ACP Agents`.
- MCP servers are still represented by local mock data in the main list even though live backend actions exist.
- ACP agents are presented as installable package cards, but the product meaning is wrong unless the UI explains they are Agent Client Protocol implementations that can be wired to ACP clients.
- Marketplace sources are over-promoted as a tab instead of a filter/source-management surface.
- Mobile controls duplicate desktop controls instead of inheriting the Gateways search-with-filter pattern.

## Data Sources

The live preview must use real backend reads.

Allowed read actions:

- `sources.list`
- `plugins.list`
- `agent.list`
- `agent.get`
- `mcp.list`
- `mcp.get`
- `mcp.versions`
- `mcp.meta.get`
- plugin detail/read actions already required by existing detail surfaces

Mutating actions such as `plugin.install`, `plugin.uninstall`, `plugin.cherry_pick`, `sources.add`, `sources.remove`, `mcp.install`, and `agent.install` must not execute from `/dev/marketplace`.

## Page Structure

### Header

Use the standard Labby shell and `AppHeader`.

Header actions:

- Refresh catalog data
- Add marketplace source, visible as a previewable action but blocked in `/dev/*` read-only mode
- Card/table density controls on desktop

Do not put a tab strip under the header.

### Hero

Use a Tier 1 Aurora panel.

Copy direction:

```text
Marketplace
Browse plugins, MCP servers, and ACP agents from one live catalog. Preview install flows safely from the read-only dev route.
```

The hero should include compact read-only context when rendered under `/dev/marketplace`.

### Summary Lenses

Use Gateway-style summary cards on desktop and compact chips on mobile.

Initial lenses:

- All
- Installed
- Plugins
- MCP servers
- ACP agents
- Sources

Selecting a lens updates filters instead of switching tabs. The lens state is part of the same filter model.

### Filter Rail

Use the Gateways page filter rail pattern.

Desktop:

- left rail, about `280px`
- search field at top
- grouped checkbox/filter controls below
- clear filters action
- sort control in the rail

Mobile:

- one search field with embedded clear and filter buttons
- active filter chips below search
- filter sheet/panel reusing the same groups

Filter groups:

- Type: plugin, MCP server, ACP agent, source
- Install state: installed, update available, not installed, built-in
- Runtime/ecosystem: Claude, Codex, Gemini, generic, MCP, ACP
- Source: marketplace source names for plugins and source records
- Distribution: npm, Python, binary, local, git, GitHub

The final set can be reduced if the live data does not support a facet yet, but unsupported facets should not be faked.

### Results Area

The results area supports two display modes.

Card mode:

- visual cards similar to current cards, but unified across item types
- cards must expose item type, source/package id, version, install state, and primary action preview
- cards should avoid large empty whitespace and should preserve the current dense catalog feel

Table/list mode:

- condensed rows for scanning large catalogs
- columns: item, type, source/package, version, state, updated/distribution, action
- row actions open detail or install-preview flows

The density switch controls this mode. It should be available in the header on desktop. On mobile, the default should be the dense single-column row/card hybrid.

### Source Management

Marketplace sources should be represented as catalog items and as a source filter group, not as a primary tab.

Clicking a source should filter the catalog to that source. Source add/remove/edit flows can be previewed from `/dev/marketplace`, but final writes must be blocked in read-only mode.

## ACP Agent Semantics

ACP agents are not generic Lab plugins. The UI must explain them as agents implementing the Agent Client Protocol that can be used with an ACP client.

Agent cards/rows should expose:

- name
- id
- version
- distribution method
- install/wiring state if known
- repository/website/license/authors if available
- primary action label that does not imply immediate activation in `/dev/*`

The install preview should communicate what installation would do: make the agent available as an ACP implementation for compatible clients. It should not claim that installing automatically adds the agent to `/chat` unless the backend actually wires that behavior.

## Read-Only `/dev/marketplace`

`/dev/marketplace` is public and unauthenticated, so the preview must be read-only.

Frontend guard:

- route read actions through the dev preview action client
- block non-whitelisted mutations before network requests
- show a clear read-only toast/message when a blocked action is attempted

Backend guard:

- `/dev/api/marketplace` must whitelist read-only marketplace actions
- non-whitelisted actions return `403` with kind `dev_preview_read_only`
- allowed actions reuse real marketplace dispatch so data is live

Interactive mutation flows should still be reviewable. The user should be able to open install/source dialogs, inspect fields, and reach the final step, but submission must be disabled or blocked with read-only messaging.

## States

Required states:

- loading skeleton for summary/filter/results
- populated catalog
- empty no-data catalog
- empty filtered result
- backend load failure
- read-only mutation blocked
- partial data failure where one inventory type fails but others load

Partial failures should not blank the entire catalog if other sections loaded successfully.

## Responsive Behavior

Desktop:

- hero
- summary strip
- two-column layout with filter rail and results
- card/table mode switch

Tablet:

- retain two-column layout only if the filter rail and result area remain readable
- otherwise collapse to mobile filter shell

Mobile:

- compact summary chips
- search-with-embedded-filter button
- active filter chips
- dense single-column results
- no duplicated desktop controls stacked under mobile controls

## Accessibility

Requirements:

- summary lenses and filter pills use `aria-pressed` where applicable
- search inputs have stable labels and names
- icon-only buttons have labels
- dialogs have clear titles/descriptions
- keyboard focus remains visible and follows Aurora focus rules
- card links and row actions have distinct accessible names
- read-only disabled actions explain why they are disabled

## Design System Alignment

Marketplace v2 must follow the design-system contract.

Required alignment:

- Aurora semantic tokens only
- `AURORA_DISPLAY_1`, `AURORA_DISPLAY_2`, `AURORA_DISPLAY_NUMBER`, and `AURORA_MUTED_LABEL` for approved typography moments
- Gateway mobile search/filter pattern
- Gateway filter rail pattern
- approved elevation tiers for hero, summary, filters, cards, tables, and inspectors
- `aurora-scrollbar` on scroll containers
- no raw hex, `rgba`, or `hsl` in product classes
- no shadcn generic color tokens in product code

## Deviations

No deviations are approved yet.

If implementation finds a cleaner or more correct approach that deviates from the design-system contract, the deviation must be presented to the user for approval and recorded here before completion.

## Review And Completion

Before completion:

- verify `/dev/marketplace` loads live data
- verify mutation actions are blocked in `/dev/marketplace`
- verify `/v1/marketplace` remains authenticated
- run desktop and mobile browser review with Chrome DevTools MCP or browser-devtools automation
- inspect console and network requests
- compare implementation against this spec
- compare implementation against the design-system contract
- address all issues found
