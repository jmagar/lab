import { cn } from '@/lib/utils'
import { AURORA_CONTROL_SURFACE, controlTone } from '@/components/aurora/tokens'

/**
 * Gateway-specific surface tokens.
 * Shared Aurora constants live in @/components/aurora/tokens.
 */

export const AURORA_GATEWAY_STAT =
  'rounded-[1.05rem] border border-aurora-border-strong bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(12,27,38,0.98))] px-4 py-3 shadow-[0_8px_18px_rgba(0,0,0,0.18),var(--aurora-highlight-medium)]'

export const AURORA_GATEWAY_CARD =
  'rounded-[1.2rem] border border-aurora-border-strong bg-[linear-gradient(180deg,rgba(14,31,44,0.98),rgba(9,21,30,0.98))] shadow-[0_16px_30px_rgba(0,0,0,0.2),var(--aurora-highlight-medium)]'

export const AURORA_GATEWAY_MUTED_CARD =
  'rounded-[1.15rem] border border-aurora-border-strong/80 bg-[linear-gradient(180deg,rgba(9,22,31,0.92),rgba(8,18,26,0.98))] shadow-[0_12px_22px_rgba(0,0,0,0.16),var(--aurora-highlight-medium)]'

export const AURORA_GATEWAY_ROW =
  'border-t border-aurora-border-strong/80 bg-[linear-gradient(180deg,rgba(14,31,44,0.72),rgba(10,22,31,0.88))] transition-colors hover:bg-[linear-gradient(180deg,rgba(17,38,54,0.9),rgba(12,27,38,0.96))]'

export const AURORA_GATEWAY_DISABLED_ROW =
  'border-t border-aurora-border-strong/60 bg-[linear-gradient(180deg,rgba(8,19,27,0.9),rgba(7,16,24,0.96))] text-aurora-text-muted'

export const AURORA_GATEWAY_SUBTLE_SURFACE =
  'rounded-[0.95rem] border border-aurora-border-strong bg-[rgba(7,17,26,0.48)] shadow-[inset_0_1px_0_rgba(255,255,255,0.025)]'

export function gatewayActionTone(variant: 'default' | 'accent' = 'default') {
  return cn('border', AURORA_CONTROL_SURFACE, controlTone(variant))
}

export function gatewayStatusTone(healthy: boolean, connected: boolean) {
  if (healthy && connected) {
    return {
      dot: 'bg-aurora-accent-strong shadow-[0_0_0_4px_rgba(103,203,250,0.08)]',
      text: 'text-aurora-accent-strong',
      label: 'Healthy',
    }
  }

  if (!connected) {
    return {
      dot: 'bg-aurora-error shadow-[0_0_0_4px_rgba(199,132,144,0.08)]',
      text: 'text-aurora-error',
      label: 'Disconnected',
    }
  }

  return {
    dot: 'bg-aurora-warn shadow-[0_0_0_4px_rgba(198,163,107,0.08)]',
    text: 'text-aurora-warn',
    label: 'Needs attention',
  }
}
