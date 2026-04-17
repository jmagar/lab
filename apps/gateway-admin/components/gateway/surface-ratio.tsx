import type { LucideIcon } from 'lucide-react'

import { cn } from '@/lib/utils'

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
        'inline-flex items-center gap-2 rounded-full border bg-background px-3 py-1.5 text-sm font-medium text-foreground',
        className,
      )}
      title={`${label}: ${exposed}/${total}`}
      aria-label={`${label}: ${exposed} of ${total}`}
    >
      <Icon className="size-4 text-muted-foreground" aria-hidden="true" />
      <span className="tabular-nums">{exposed}/{total}</span>
    </div>
  )
}
