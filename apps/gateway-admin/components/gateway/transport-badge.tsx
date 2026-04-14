import { Globe, Terminal } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { TransportType } from '@/lib/types/gateway'

interface TransportBadgeProps {
  transport: TransportType
  className?: string
}

export function TransportBadge({ transport, className }: TransportBadgeProps) {
  const isHttp = transport === 'http'
  
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-md px-2 py-0.5 text-xs font-medium',
        isHttp 
          ? 'bg-[#00b0ff]/20 text-[#00b0ff] border border-[#00b0ff]/50 shadow-sm shadow-[#00b0ff]/20' 
          : 'bg-[#ea80fc]/20 text-[#ea80fc] border border-[#ea80fc]/50 shadow-sm shadow-[#ea80fc]/20',
        className
      )}
    >
      {isHttp ? <Globe className="size-3" /> : <Terminal className="size-3" />}
      {isHttp ? 'HTTP' : 'stdio'}
    </span>
  )
}
