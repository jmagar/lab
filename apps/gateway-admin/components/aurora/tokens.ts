import type { LogLevel } from '@/lib/types/logs'

export const AURORA_LEVEL_TEXT: Record<LogLevel, string> = {
  trace: 'text-aurora-text-muted',
  debug: 'text-aurora-accent-strong',
  info: 'text-aurora-accent-primary',
  warn: 'text-aurora-warn',
  error: 'text-aurora-error',
}

export const AURORA_DISPLAY_TITLE = 'font-display tracking-[-0.02em]'
export const AURORA_DISPLAY_NUMBER = 'font-display tracking-[-0.04em] tabular-nums'

// Page-level h1 — Manrope, 34px, -0.045em tracking.
export const AURORA_DISPLAY_1 =
  'font-display text-[34px] leading-[1.1] tracking-[-0.045em] text-aurora-text-primary'

// Section-level h2 — alias to AURORA_DISPLAY_TITLE for semantic clarity.
export const AURORA_DISPLAY_2 = AURORA_DISPLAY_TITLE

export const AURORA_PAGE_FRAME =
  'mx-auto flex w-full max-w-[1740px] flex-col gap-5 px-4 py-6 sm:px-6 xl:px-8'

export const AURORA_PAGE_SHELL =
  'bg-[radial-gradient(circle_at_top_left,rgba(41,182,246,0.08),transparent_26%),radial-gradient(circle_at_top_right,rgba(103,203,250,0.06),transparent_20%)]'

export const AURORA_MEDIUM_PANEL =
  'rounded-[1.35rem] border border-aurora-border-strong bg-aurora-panel-medium shadow-[var(--aurora-shadow-medium),var(--aurora-highlight-medium)]'

export const AURORA_STRONG_PANEL =
  'rounded-[1.4rem] border border-aurora-border-strong bg-aurora-panel-strong shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong)]'

export const AURORA_CONTROL_SURFACE =
  'rounded-[0.95rem] border-aurora-border-strong bg-aurora-control-surface shadow-[0_8px_16px_rgba(0,0,0,0.16),var(--aurora-highlight-medium)]'

export const AURORA_MUTED_LABEL = 'text-[11px] font-medium uppercase tracking-[0.18em] text-aurora-text-muted'

export const AURORA_TAIL_ROW =
  'grid grid-cols-[170px_84px_130px_minmax(0,1fr)] gap-3'

export const AURORA_MESSAGE_SURFACE =
  'rounded-[1rem] border border-aurora-border-strong bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] shadow-[0_8px_16px_rgba(0,0,0,0.16),var(--aurora-highlight-medium)]'

export function pillTone(active: boolean): string {
  return active
    ? 'border-aurora-accent-primary/28 bg-[linear-gradient(180deg,rgba(16,35,48,0.96),rgba(11,25,35,0.98))] text-aurora-text-primary shadow-[var(--aurora-active-glow)]'
    : 'border-aurora-border-strong bg-[linear-gradient(180deg,rgba(10,23,32,0.92),rgba(9,20,28,0.98))] text-[#b9ced9] shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]'
}

export function controlTone(variant: 'default' | 'accent' = 'default'): string {
  return variant === 'accent'
    ? 'border-aurora-accent-primary/28 bg-[linear-gradient(180deg,rgba(16,35,48,0.96),rgba(11,25,35,0.98))] text-aurora-accent-strong shadow-[var(--aurora-active-glow)]'
    : 'border-aurora-border-strong bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] text-[#d6eaf3] shadow-[0_8px_16px_rgba(0,0,0,0.16),var(--aurora-highlight-medium)]'
}
