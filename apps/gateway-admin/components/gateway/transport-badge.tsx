import { Globe, Layers, Terminal } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { TransportType } from '@/lib/types/gateway'

interface TransportBadgeProps {
  transport: TransportType
  className?: string
}

export function TransportBadge({ transport, className }: TransportBadgeProps) {
  const config = (() => {
    switch (transport) {
      case 'http':
        return {
          label: 'HTTP',
          className:
            'border-aurora-accent-primary/28 bg-[linear-gradient(180deg,rgba(16,35,48,0.96),rgba(11,25,35,0.98))] text-aurora-accent-strong shadow-[var(--aurora-active-glow)]',
          icon: Globe,
        }
      case 'stdio':
        return {
          label: 'stdio',
          className:
            'border-aurora-border-strong bg-[linear-gradient(180deg,rgba(17,32,44,0.98),rgba(11,22,30,0.98))] text-[#d2c8e8] shadow-[0_8px_16px_rgba(0,0,0,0.14),var(--aurora-highlight-medium)]',
          icon: Terminal,
        }
      case 'lab_service':
        return {
          label: 'Lab',
          className:
            'border-aurora-border-strong bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] text-[#cae5cf] shadow-[0_8px_16px_rgba(0,0,0,0.14),var(--aurora-highlight-medium)]',
          icon: Layers,
        }
      default: {
        const exhaustive: never = transport
        return exhaustive
      }
    }
  })()

  const Icon = config.icon
  
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] font-medium uppercase tracking-[0.16em]',
        config.className,
        className
      )}
    >
      <Icon className="size-3" />
      {config.label}
    </span>
  )
}
