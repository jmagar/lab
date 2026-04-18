import type { LucideIcon } from 'lucide-react'

import { cn } from '@/lib/utils'
import { AURORA_GATEWAY_SUBTLE_SURFACE } from './gateway-theme'

interface SurfaceRatioProps {
  icon: LucideIcon
  label: string
  exposed: number
  total: number
  className?: string
}

export function SurfaceRatio({ icon: Icon, label, exposed, total, className }: SurfaceRatioProps) {
  return (
    <div
      className={cn(
        AURORA_GATEWAY_SUBTLE_SURFACE,
        'inline-flex items-center gap-2 px-3 py-2 text-sm font-medium text-aurora-text-primary',
        className,
      )}
      title={`${label}: ${exposed}/${total}`}
      aria-label={`${label}: ${exposed} of ${total}`}
    >
      <Icon className="size-4 text-aurora-accent-strong" aria-hidden="true" />
      <span className="tabular-nums">{exposed}/{total}</span>
      <span className="text-[11px] uppercase tracking-[0.16em] text-aurora-text-muted">{label}</span>
    </div>
  )
}
