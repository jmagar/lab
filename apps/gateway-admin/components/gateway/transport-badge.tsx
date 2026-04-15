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
            'bg-[#00b0ff]/20 text-[#00b0ff] border border-[#00b0ff]/50 shadow-sm shadow-[#00b0ff]/20',
          icon: Globe,
        }
      case 'stdio':
        return {
          label: 'stdio',
          className:
            'bg-[#ea80fc]/20 text-[#ea80fc] border border-[#ea80fc]/50 shadow-sm shadow-[#ea80fc]/20',
          icon: Terminal,
        }
      case 'lab_service':
        return {
          label: 'Lab',
          className:
            'bg-[#8bc34a]/20 text-[#8bc34a] border border-[#8bc34a]/50 shadow-sm shadow-[#8bc34a]/20',
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
        'inline-flex items-center gap-1.5 rounded-md px-2 py-0.5 text-xs font-medium',
        config.className,
        className
      )}
    >
      <Icon className="size-3" />
      {config.label}
    </span>
  )
}
