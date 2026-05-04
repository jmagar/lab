---
name: aurora-checklist
description: Authoritative checklist for reviewing apps/web/ changes — fully aligned with docs/design/design-system-contract.md. Covers Aurora tokens, typography ramp, elevation, radius, scrollbar, import rules, shadcn bans, TanStack Query v5, logging, and auth. Preloaded by aurora-reviewer.
user-invocable: false
---

Source of truth: `docs/design/design-system-contract.md`. When in doubt, that file wins.

---

## 1. Color tokens — hard invariant

- No raw hex, `rgba(`, or `hsl(` in `className`, inline styles, or `.css` files
- All colors must come from `var(--aurora-*)` tokens — **not** `var(--color-*)`
- **Sanctioned exceptions only:**
  - `lib/branding/service-brands.ts` — third-party brand colors (Plex, Sonarr, Radarr, etc.)
  - `--aurora-preview-allowed/unmatched/highlight` — only inside `components/gateway/exposure-policy-editor.tsx`
  - Any other raw hex/rgba is a violation

## 2. Shadcn generic tokens banned in product code

Product code (`app/**`, `components/**` except `components/ui/**`) must not use shadcn tokens. Flag these and require the Aurora equivalent:

| Banned | Required instead |
|---|---|
| `text-muted-foreground` | `text-aurora-text-muted` |
| `text-foreground` | `text-aurora-text-primary` |
| `bg-card`, `bg-card/NN` | `bg-aurora-panel-medium` or `bg-aurora-panel-strong` |
| `bg-background` | `bg-aurora-page-bg` |
| `bg-muted` | `bg-aurora-control-surface` |
| `border-border`, `border-border/NN` | `border-aurora-border-strong` |

`components/ui/**` primitives may still use shadcn tokens — they must stay syncable with upstream.

## 3. Typography — display slot assignments

**Most common drift source.** Each display moment has a required token:

| Location | Required token | Violation signal |
|---|---|---|
| Route-level `<h1>` | `AURORA_DISPLAY_1` + `text-aurora-text-primary` | `<h1>` without Display token |
| Section / panel headings | `AURORA_DISPLAY_2` (alias: `AURORA_DISPLAY_TITLE`) | heading without ramp token |
| Large metric / stat numbers | `AURORA_DISPLAY_NUMBER` + tabular numerals | bare `text-2xl font-bold` on a number |
| Eyebrow / category label | `AURORA_MUTED_LABEL` | `text-xs uppercase tracking-*` combination |
| Badge / chip label | `AURORA_BADGE_LABEL` | tiny badge with hand-rolled sizing |
| Dense metadata | `AURORA_DENSE_META` | `text-[11px]` or `text-xs font-medium` for metadata |
| Compact panel title | `AURORA_COMPACT_TITLE` | |

Rules:
- Do **not** override a ramp token's size, weight, or tracking with additional Tailwind classes — color is the only permitted companion class
- `Manrope` for display family; `Inter` for working UI (navigation, buttons, forms, tables, logs)
- Hand-rolled `text-xs uppercase tracking-[...]` is always a drift signal — must use `AURORA_MUTED_LABEL`

## 4. Elevation tiers

| Tier | Use | Token(s) |
|---|---|---|
| 0 | Page canvas | `bg-aurora-page-bg` |
| 1 | Toolbars, header strips, secondary panels | `--aurora-shadow-medium`, `--aurora-highlight-medium` |
| 2 | Primary data panels, inspectors, metric surfaces | `--aurora-shadow-strong`, `--aurora-highlight-strong` |

- `Card variant="strong"` = Tier 2; `variant="medium"` = Tier 1
- Inspectors must use Tier 2
- Do not flatten all surfaces to a single visual plane

## 5. Radius contract

| Token | Value | Tailwind | Use |
|---|---|---|---|
| `--radius-1` | 14px | `rounded-aurora-1` | dense controls, small buttons |
| `--radius-2` | 18px | `rounded-aurora-2` | inline cards, metadata blocks |
| `--radius-3` | 22px | `rounded-aurora-3` | major panels, toolbars, inspectors |
| pill | 999px | `rounded-full` | filter chips, compact badges |

- New code must use `rounded-aurora-1/2/3` before any arbitrary value
- One legacy exception: `rounded-[1rem]` (16px) may remain; do not introduce new arbitrary radii
- `Button` defaults to `rounded-md` — intentional, do not change

## 6. Scrollbar

Every `overflow-auto`, `overflow-scroll`, `overflow-y-auto`, or `overflow-x-auto` container must include the `aurora-scrollbar` utility class. Missing `aurora-scrollbar` is a violation.

## 7. Import rules

- All `components/ui/**` imports must use the `@/` alias — relative imports (`../../components/ui/...`) are banned
- Exception: auth surfaces under `components/auth/**` or `app/**/auth/**` imported by `node:test` harnesses may inline Button's class string; must stay on Aurora tokens and include an explanatory comment

## 8. Theme / dark mode

- No `dark:` selectors with one-off colors — Aurora token system already resolves for both modes
- New tokens added to dark mode must be paired with a light-mode value in the same commit
- No `bg-gradient-to-*` with shadcn tokens on auth surfaces — use `AURORA_PAGE_SHELL`

## 9. Active / hover / focus states

- Focus ring must use `--aurora-focus-ring` token (`rgba(41, 182, 246, 0.34)`)
- Hover backgrounds use `bg-aurora-hover-bg` utility — never the raw `#17364b` hex
- No glossy fills, bright radial sheen, or full-accent flooded control backgrounds for standard selection

## 10. Motion

- Allow: hover transitions, expand/collapse, loading indicators
- Forbidden: decorative ambient motion, large animated glows, animated gradients in normal control states

## 11. Accessibility

- All interactive controls require visible `focus-visible` treatment
- Contrast must remain readable on dark surfaces (WCAG AA, ≥4.5:1)
- Inputs require accessible names: `label`, `aria-label`, or both
- Product forms need a stable `name` attribute

## 12. Next.js App Router

- Only `app/` routing — no `pages/` directory
- `'use client'` only when the component uses browser APIs or React hooks
- No Pages Router patterns (`getServerSideProps`, `getStaticProps`)

## 13. TanStack Query v5

- `useQuery` / `useMutation` use `queryKey` + `queryFn` shape
- `queryFn` **throws** `ApiError` — `new ApiError(error.kind, error.message, error.status)`; does not return an error value
- `isPending` not `isLoading` (v5 rename)
- All API calls through the `openapi-fetch` typed client — never raw `fetch`

## 14. TypeScript / Biome

- No `any` without an explanatory comment
- `cn()` for conditional classNames — no string concatenation
- Biome-clean: no unused imports, consistent quotes

## 15. Logging

- Client errors via `logClientError()` from `@/lib/log.ts`
- Field names from `LOG_FIELDS` / `EVENTS` in `@/lib/log-fields.ts` — no inline strings

## 16. Auth

- Auth state from `@/lib/auth.ts` only — not localStorage, sessionStorage, or manual cookie parsing

---

## Output format

Group violations by section number + name. Include `file:line` for each. Mark anything unverifiable as `UNVERIFIABLE`. End with a single verdict: **PASS**, **PASS with warnings**, or **FAIL**.
