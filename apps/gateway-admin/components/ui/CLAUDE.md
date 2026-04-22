# `components/ui/` — shadcn primitive customization rule

These are forked shadcn primitives. Keep them shallow so re-syncing with upstream stays trivial.

## What may be baked into a primitive

- **Brand identity tokens**: font family, weight, tracking, color tokens (`font-display`, `text-aurora-*`, `bg-aurora-*`).
- **Default variant selection**: e.g. `cardVariants` mapping Aurora tier names.

Example: `CardTitle` defaults to `font-display` because every Aurora card header should use Manrope. Forgetting it at the call site is a contract violation; baking it in is safe-by-default.

## What must NOT be baked in

- **Layout / spacing**: `gap-`, `p-`, `m-`, `grid-cols-*`. These are call-site decisions.
- **Sizing / type ramp**: `text-xl`, `text-3xl`, `h-`, `w-`. Use `AURORA_DISPLAY_*` token strings at the call site instead.
- **Behavioral coupling**: callbacks, refs, anything that changes the primitive's contract.

Rule of thumb: if the change describes *what the brand looks like*, bake it in. If it describes *where this instance sits in this layout*, leave it for the caller.
