# Labby Design System Contract

**Status:** Active  
**Scope:** `apps/gateway-admin` web UI  
**Mode:** Dark (primary) and light  
**Primary reference sandbox:** `/design-system`

## Purpose

This document is the root-level source of truth for the Labby web UI design system.

It defines the visual and interaction contract that new pages, refactors, and component work must follow. The goal is to keep Labby consistent as a premium control plane: clean, modern, and operator-friendly without feeling glossy, intimidating, or over-styled.

This contract is implementation-facing. It exists so product pages do not drift into one-off styling decisions.

Related documents:

- [Aurora Dark Theme Spec](./superpowers/specs/2026-04-17-aurora-dark-theme-spec.md)
- [Design System Page Plan](./superpowers/plans/2026-04-17-design-system-page.md)

## Theme Modes

Aurora is designed dark-first. A light remap of the same token system is also supported so users can switch via the theme toggle in the sidebar footer (`next-themes` system/light/dark).

Rules:

- the canonical visual reference is dark; design and review work happens against dark first
- the light remap lives in `app/globals.css` under `.light` and overrides only the raw Aurora variables; semantic tokens in `:root` already resolve correctly
- both modes must stay on the same semantic-token surface — do not branch on `dark:` selectors with one-off colors when the Aurora system already provides a token
- any new Aurora token added to dark mode must be paired with a light-mode value in the same commit

## Product Direction

Labby uses the `Aurora` theme language.

Aurora should feel:

- premium, but not glossy
- modern, but not flashy
- calm, but not dull
- dense enough for operator workflows, but not daunting

Aurora should avoid:

- bright neon dark-mode styling
- heavy gradient fills for standard controls
- high-alert warning and error colors used by default
- a field of identical flat cards
- decorative motion that competes with operational data

## Typography

Labby uses a split typography system.

- Display family: `Manrope`
- Working UI family: `Inter`

### `Manrope` Usage

Use `Manrope` for:

- page titles
- section headers
- top-level metric numbers

### `Inter` Usage

Use `Inter` for:

- navigation labels
- buttons and controls
- forms
- tables
- logs and dense operational rows
- metadata and inspectors
- body copy

### Typography Ramp

When a call site uses an `AURORA_DISPLAY_*` token, the token is the **complete** specification. Do not override its size (`text-2xl`, `text-3xl`), weight (`font-semibold`, `font-bold`), or tracking (`tracking-tight`) via additional classes — Tailwind's merge order will silently win and your "token-based" typography will render as something else entirely. Color is the only permitted companion class (e.g. `text-aurora-text-primary`, `text-aurora-warn`). If you need a size the ramp doesn't cover, add a new ramp slot in this document and in `components/aurora/tokens.ts`; do not patch it per call site.


#### Display 1

- family: `Manrope`
- size: `34px`
- line-height: `1.04`
- weight: `800`
- tracking: `-0.045em`
- use: major page titles

#### Display 2

- family: `Manrope`
- size: `19px`
- line-height: `1.12`
- weight: `700`
- tracking: `-0.02em`
- use: section headers

#### Metric Display

- family: `Manrope`
- size: `28px`
- line-height: `1`
- weight: `800`
- tracking: `-0.04em`
- use: top-level metric numbers
- use tabular numerals when possible

#### Body

- family: `Inter`
- size: `14px`
- line-height: `1.55`
- weight: `400`
- use: standard copy, helper text

#### Control

- family: `Inter`
- size: `13px`
- line-height: `1.2`
- weight: `500` to `600`
- use: buttons, inputs, selects, pills

#### Dense Data

- family: `Inter` or mono where required
- size: `12px` to `13px`
- line-height: `1.35` to `1.5`
- use: tables, logs, inspectors

#### Eyebrow

- family: `Inter`
- size: `10px` to `11px`
- line-height: `1`
- weight: `700`
- tracking: `0.14em` to `0.18em`
- use uppercase

The single sanctioned Eyebrow implementation is `AURORA_MUTED_LABEL`. Do **not** hand-roll eyebrow styling with `text-xs uppercase tracking-*` — the 1px size delta, the 500-vs-700 weight gap, and the arbitrary tracking value all compound into visible inconsistency. A `text-xs uppercase tracking-[...]` combination in product code is a drift signal and must be migrated.

## Color Contract

All new page and component styling should use semantic Aurora tokens rather than embedded one-off hex values.

### Base Surfaces

- `--aurora-page-bg: #07131c`
- `--aurora-nav-bg: #07111a`
- `--aurora-panel-medium: #102330`
- `--aurora-panel-medium-top: rgba(20, 44, 60, 0.18)`
- `--aurora-panel-strong: #13293a`
- `--aurora-panel-strong-top: #173245`
- `--aurora-control-surface: #0c1a24`
- `--aurora-control-surface-top: rgba(18, 40, 56, 0.96)`

### Borders And Text

- `--aurora-border-default: #1d3d4e`
- `--aurora-border-strong: #24536c`
- `--aurora-text-primary: #e6f4fb`
- `--aurora-text-muted: #a7bcc9` — lifted from the original `#90a9b9` to clear WCAG AA (≥4.5:1) on `--aurora-panel-medium`. The lighter value is the source of truth.

### Accent Family

- `--aurora-accent-primary: #29b6f6`
- `--aurora-accent-strong: #67cbfa`
- `--aurora-accent-deep: #1c7fac`

### State Colors

- `--aurora-warn: #c6a36b`
- `--aurora-error: #c78490`
- `--aurora-success: #7dd3c7` — muted teal in the accent family; used for success/connected/authorized states. Reads calm, not neon.
- `--aurora-focus-ring: rgba(41, 182, 246, 0.34)`
- `--aurora-active-glow: 0 0 0 1px rgba(41, 182, 246, 0.18), 0 0 16px rgba(41, 182, 246, 0.08)`

### Interaction Helper Tokens

- `--aurora-hover-bg: #17364b` — single source for hover backgrounds on outline/ghost controls. Use the `bg-aurora-hover-bg` utility, never the raw hex.

### Live-Preview Tokens (restricted scope)

These tokens intentionally sit brighter than the Aurora muted family. They exist so a real-time visualization surface can signal `allowed` / `unmatched` / `highlight` states at a glance. They are **not** general-purpose status colors.

- `--aurora-preview-allowed: #00e676`
- `--aurora-preview-unmatched: #ff9100`
- `--aurora-preview-highlight: #ffea00`

Sanctioned use:

- the exposure-policy editor's live-preview surface (`components/gateway/exposure-policy-editor.tsx`)

These are the only place in Labby where saturated/neon color and decorative gradients are allowed. Any new use must be added to this list — do not reach for these tokens for routine status display.

### Status Guidance

Status accents must stay muted and integrated into the Aurora family.

- success/live should read clear, not neon
- warning should read informational before alarming
- error should read serious without becoming the loudest surface on the page by default

## Elevation And Surface Contract

Aurora uses noticeable lift with tiers.

### Tier 0

- the page canvas
- deepest navy surface
- should clearly sit behind all working UI

### Tier 1

Use for:

- toolbars
- header control strips
- secondary support panels

Supporting tokens:

- `--aurora-shadow-medium: 0 12px 24px rgba(0, 0, 0, 0.18)`
- `--aurora-highlight-medium: inset 0 1px 0 rgba(255, 255, 255, 0.035)`

### Tier 2

Use for:

- primary data panels
- inspectors
- strong metric surfaces
- main stream containers

Supporting tokens:

- `--aurora-shadow-strong: 0 20px 38px rgba(0, 0, 0, 0.26)`
- `--aurora-highlight-strong: inset 0 1px 0 rgba(255, 255, 255, 0.05)`

### Rule

Do not flatten major surfaces into a single visual plane. Toolbar/header layers should be calmer than main content layers.

## Radius Contract

Aurora uses a token-backed radius scale exposed as both CSS variables and Tailwind utilities.

| Token | Value | Tailwind utility | Use |
|-------|-------|------------------|-----|
| `--radius-1` | `14px` | `rounded-aurora-1` | dense controls, small buttons |
| `--radius-2` | `18px` | `rounded-aurora-2` | inline cards, metadata blocks |
| `--radius-3` | `22px` | `rounded-aurora-3` | major panels, toolbars, inspectors |
| `--radius-pill` | `999px` | `rounded-full` | filter chips, compact state badges |

Rules:

- new code must reach for the tokens (`rounded-aurora-1/2/3`) before any arbitrary value
- one remaining arbitrary radius is tolerated as a **legacy tuned variant**: `rounded-[1rem]` (16px) sits between Radius 1 and 2 with no clean home and may be migrated to `rounded-aurora-2` incrementally
- `Button` defaults to `rounded-md` (~8px) for compact dense controls — this is below Radius 1 by design and stays as-is
- do not introduce new arbitrary radii outside the table above without updating this contract

## Spacing Contract

Aurora uses a compact operator spacing system.

- `4px` for micro alignment and icon gaps
- `8px` for tight internal spacing
- `10px` for compact control grouping
- `12px` for default control padding and short stacks
- `16px` for standard card/panel padding
- `20px` for major group spacing
- `24px` for section separation

### Rule

Favor compact clarity over large breathable marketing spacing. Labby is an operational UI.

## Active, Hover, And Focus Rules

Aurora uses restrained active states.

State should be communicated through:

- border change
- text emphasis
- indicator dots or check marks
- slight surface deepening
- subtle outer glow

Aurora should avoid:

- glossy selected fills
- bright radial sheen
- full-accent flooded control backgrounds for standard selection

### Focus

Interactive elements must present a clear focus-visible treatment using the shared Aurora focus ring language.

### Motion

Motion should be minimal and functional.

Allow:

- hover transitions
- expand/collapse transitions
- loading and progress indicators

Avoid:

- decorative ambient motion
- large animated glows
- animated gradients in normal control states

## Component Contract

### Buttons

Labby uses a button hierarchy:

- primary
- secondary
- outline
- ghost
- destructive

Rules:

- primary actions should use the Aurora accent family with restrained fill and clear contrast
- secondary and outline actions should rely on surface, border, and text hierarchy rather than louder color
- destructive actions should remain clearly distinct, but still muted within Aurora
- buttons should not introduce one-off shadows, radii, or typography

### Pills And Multi-Select Filters

Pill-style checkbox controls are part of the system.

Rules:

- default state should stay flat and calm
- selected state should use border emphasis, a subtle glow, and a clear internal indicator
- selected pills should not use bright sheen or heavy filled gradients

### Inputs And Selects

Rules:

- use Aurora control surfaces
- keep borders and focus states consistent with the shared token system
- validation states must layer on top of the control contract rather than replace it with unrelated styling

### Status Badges

Status badges should be compact and quiet.

Rules:

- badges should reinforce meaning without becoming the main visual event
- keep warning and error badges muted
- do not use highly saturated fills for routine status display

### Tables And Dense Data

Labby supports dense operational views.

Rules:

- favor scanability over decorative row styling
- keep row heights compact and consistent
- use truncation by default where scanability matters
- allow wrapping only when the interaction clearly calls for expansion
- use mono only where value alignment or log readability benefits from it

### Panels And Inspectors

Rules:

- inspectors should use Tier 2 lift
- secondary support panels should use Tier 1 lift unless they are the primary interaction surface
- avoid mixing many panel languages on a single page

### Cards (`components/ui/card.tsx`)

`CardTitle` defaults to `font-display tracking-[-0.02em]` so card headers automatically pick up Manrope and the Aurora display tracking. Sizing stays the caller's responsibility — compose `AURORA_DISPLAY_2` (or another ramp entry) when an explicit ramp slot is required.

`Card` exposes `variant="medium" | "strong"` mapped to Tier 1 / Tier 2. Use `variant="strong"` for the primary surface on a page; everything else stays on the medium default.

### Service Brand Identity

External-service brand colors (Plex orange, Sonarr cyan, Radarr yellow, etc.) are kept in `lib/branding/service-brands.ts`, not inlined in components. They are the single sanctioned exception to the "no raw hex" rule because they encode third-party brand identity rather than Aurora UI tone.

### Empty, Loading, Success, Warning, Error States

Rules:

- these states must feel like part of the same product, not separate illustration cards
- empty states should stay concise and operational
- loading states should be understated
- success/warning/error states should communicate clearly without breaking palette discipline

## Display Slot Assignments

The Typography ramp defines sizes; this section defines which component types own which ramp slot.

| Component | Required token | Notes |
|---|---|---|
| Route-level `<h1>` (top of an admin page, login screen, etc.) | `AURORA_DISPLAY_1` composed with `text-aurora-text-primary` | One Display 1 per route, always. |
| Section / panel headings inside a page | `AURORA_DISPLAY_2` (or `AURORA_DISPLAY_TITLE` legacy alias) | Panel inspectors, sidebar section titles. |
| Large metric / stat card numbers | `AURORA_DISPLAY_NUMBER` | Dashboard stats, summary strips. Use tabular numerals. |
| Category / metadata label above a title | `AURORA_MUTED_LABEL` | See Eyebrow spec. |
| Card titles rendered through `CardTitle` | Default from the primitive (`font-display tracking-[-0.02em]`); compose an explicit ramp slot when you need a specific size | See Component Contract → Cards. |

A route-level `<h1>` without a Display token is a contract violation. A metric number without `AURORA_DISPLAY_NUMBER` is a contract violation. These two rules are the most common drift source in the current codebase.

## Page-Level Patterns

### Chat And Agent Sessions

The chat surface is a transcript-first product surface.

Rules:

- the primary interaction is a single conversation column, not a split transcript/activity workspace
- reasoning appears inline as a collapsible assistant-owned block
- tool and agent actions appear inline as a compact connected flow that reads as part of the assistant turn
- raw tool-call payloads are secondary detail revealed on expansion, not the primary visual treatment
- on narrow screens, session lists and other secondary navigation collapse into drawers or sheets rather than permanently consuming horizontal space

### Logs

The logs page establishes the first Aurora reference implementation for:

- dense tail-style data presentation
- lifted toolbar plus stronger stream/inspector surfaces
- structured operator panels
- restrained control states

### Gateways

The Gateways page should align with the same Aurora principles while preserving its dense table workflow:

- Aurora summary strip
- Aurora filters and pills
- Tiered panel hierarchy
- calmer status accents

### Authentication Surfaces

Login, re-auth, and auth-error screens are product surfaces, not a separate marketing shell. They must:

- use `AURORA_PAGE_SHELL` for the backdrop — no bespoke gradients, no `bg-gradient-to-*` with shadcn tokens
- use `AURORA_STRONG_PANEL` for the centered card (Tier 2 lift — same language as inspectors)
- follow the Typography ramp: Display 1 for the title, Body for intro copy, `AURORA_MUTED_LABEL` for the category eyebrow
- route the primary action through the `Button` primitive with `size="lg"`

If the auth surface lives under a dedicated auth path such as `components/auth/**` or `app/**/auth/**` and is imported by a non-Vite test runner (`node:test` + `react-dom/server`) that cannot resolve the `@/` alias, duplicate the `Button` primitive's default+lg class string inline rather than inventing styling. The inline copy must stay on pure Aurora tokens, must preserve the `AURORA_PAGE_SHELL`, `AURORA_STRONG_PANEL`, and `AURORA_MUTED_LABEL` contract around it, and must be flagged with a one-line comment explaining why it bypasses the primitive.

### Design System Sandbox

`/design-system` is the interactive implementation reference for the current web UI.

It should remain:

- reachable by direct URL only
- excluded from the primary sidebar
- safe to interact with using local fake state
- broad enough to exercise the UI patterns currently used in the app

Its sections should cover:

- foundations
- controls
- feedback
- navigation
- command palette
- data display
- application patterns

The command-palette section is the canonical reference for Labby's global `cmd+k` interaction:

- one ranked stack for destinations, actions, and recent context
- Aurora-aligned focus, density, tags, and preview treatment
- local fake state only, with no requirement for live backend search or execution

## Accessibility And Interaction

The design system must preserve usability under keyboard and assistive interaction.

Minimum rules:

- all interactive controls require visible focus treatment
- contrast must remain readable on dark surfaces
- pill filters and custom controls must remain understandable without relying on color alone
- destructive actions must stay explicit in both label and style
- dense views must preserve clear hit targets and readable text sizing

## Responsive Rules

Labby is desktop-first but must remain usable on narrower screens.

Rules:

- preserve the same system language on mobile rather than inventing alternate styling
- allow inspectors and secondary panels to collapse into sheets or drawers
- prioritize operational readability over perfectly preserving desktop density

## Engineering Rules

When implementing or refactoring UI:

- use shared semantic tokens before adding new hardcoded values
- prefer extending shared component recipes over page-local one-offs
- keep display typography to the approved display moments
- do not introduce alternate dark themes on a per-page basis
- update `/design-system` when adding or materially changing a shared interaction pattern
- treat this document as the stable contract and dated exploration docs as supporting material
- all `overflow-auto`, `overflow-scroll`, `overflow-y-auto`, and `overflow-x-auto` containers must include the `aurora-scrollbar` utility class — this applies the token-backed thin scrollbar style (defined in `app/globals.css`) consistently across all scroll surfaces
- **shadcn-generic tokens are reserved for `components/ui/` primitives.** Product code — anything under `app/**`, `components/**` except `components/ui/**` — must use the Aurora semantic equivalents:

  | Banned in product code | Use instead |
  |---|---|
  | `text-muted-foreground` | `text-aurora-text-muted` |
  | `text-foreground` | `text-aurora-text-primary` |
  | `bg-card`, `bg-card/NN` | `bg-aurora-panel-medium` or `bg-aurora-panel-strong` (choose by elevation tier) |
  | `bg-background` | `bg-aurora-page-bg` |
  | `bg-muted` | `bg-aurora-control-surface` |
  | `border-border`, `border-border/NN` | `border-aurora-border-strong` |

  The shadcn tokens exist so the forked primitives stay syncable with upstream; product code must not inherit them. The light-mode remap in `.light` pairs Aurora variables only — shadcn tokens drift in light mode even when they look right in dark.

- **primitive imports must use the `@/` alias.** Relative imports into `components/ui/**` from product code are banned so the import path doesn't encode directory structure. The one exception is auth surfaces under `components/auth/**` or `app/**/auth/**` when imported by `node:test` harnesses; see the Authentication Surfaces section for the exact escape hatch, Aurora-token-only requirement, and required inline comment.

- **no raw hex, rgba, or hsl values in className or inline styles.** All colors come from Aurora tokens. The sole sanctioned exceptions are listed in the Live-Preview Tokens section and the Service Brand Identity section.

## Approval Rule

A page is not considered aligned with the design system unless it satisfies all of the following:

- uses the Aurora token system
- follows the typography split correctly
- uses the approved elevation hierarchy
- uses restrained active states
- preserves operator-first density and readability
- does not introduce an incompatible component language

When in doubt, update the contract and the `/design-system` sandbox together rather than letting implementation drift.
