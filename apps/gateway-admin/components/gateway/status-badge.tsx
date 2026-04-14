import { cn } from '@/lib/utils'

interface StatusBadgeProps {
  healthy: boolean
  connected: boolean
  className?: string
}

export function StatusBadge({ healthy, connected, className }: StatusBadgeProps) {
  const status = healthy && connected ? 'healthy' : !connected ? 'disconnected' : 'unhealthy'
  
  const statusConfig = {
    healthy: {
      label: 'Healthy',
      className: 'bg-[#00e676]/20 text-[#00e676] border-[#00e676]/50 shadow-sm shadow-[#00e676]/20',
      dotClassName: 'bg-[#00e676] animate-pulse shadow-sm shadow-[#00e676]',
    },
    unhealthy: {
      label: 'Unhealthy',
      className: 'bg-[#ff9100]/20 text-[#ff9100] border-[#ff9100]/50 shadow-sm shadow-[#ff9100]/20',
      dotClassName: 'bg-[#ff9100] animate-pulse',
    },
    disconnected: {
      label: 'Disconnected',
      className: 'bg-[#ff1744]/20 text-[#ff1744] border-[#ff1744]/50 shadow-sm shadow-[#ff1744]/20',
      dotClassName: 'bg-[#ff1744]',
    },
  }

  const config = statusConfig[status]

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-xs font-medium',
        config.className,
        className
      )}
    >
      <span className={cn('size-1.5 rounded-full', config.dotClassName)} />
      {config.label}
    </span>
  )
}
