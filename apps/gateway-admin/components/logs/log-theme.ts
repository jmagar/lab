import type { LogLevel } from '@/lib/types/logs'

/**
 * Log-level text color classes.
 * Logs-specific: stays in this module.
 */
export const AURORA_LEVEL_TEXT: Record<LogLevel, string> = {
  trace: 'text-aurora-text-muted',
  debug: 'text-aurora-accent-strong',
  info: 'text-aurora-accent-primary',
  warn: 'text-aurora-warn',
  error: 'text-aurora-error',
}

/**
 * Log tail row grid layout.
 * Logs-specific: stays in this module.
 */
export const AURORA_TAIL_ROW =
  'grid grid-cols-[170px_84px_130px_minmax(0,1fr)] gap-3'
