import { AlertTriangle } from 'lucide-react'
import { cn } from '@/lib/utils'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import type { GatewayWarning } from '@/lib/types/gateway'

interface WarningsPillProps {
  warnings: GatewayWarning[]
  className?: string
}

export function WarningsPill({ warnings, className }: WarningsPillProps) {
  if (warnings.length === 0) return null

  const leadWarning = warnings[0]

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <span
            className={cn(
              'inline-flex items-center gap-1.5 rounded-full border border-aurora-warn/28 bg-[linear-gradient(180deg,rgba(51,41,22,0.42),rgba(30,24,14,0.56))] px-2.5 py-1 text-[11px] font-medium uppercase tracking-[0.16em] text-aurora-warn shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]',
              className
            )}
          >
            <AlertTriangle className="size-3" />
            {warnings.length} issue{warnings.length === 1 ? '' : 's'}
          </span>
        </TooltipTrigger>
        <TooltipContent side="top" className="max-w-xs">
          <div className="space-y-1">
            {leadWarning && (
              <p className="text-xs font-medium text-foreground">{leadWarning.code}</p>
            )}
            {warnings.slice(0, 3).map((warning, index) => (
              <p key={index} className="text-xs">
                {warning.message}
              </p>
            ))}
            {warnings.length > 3 && (
              <p className="text-xs text-aurora-text-muted">
                +{warnings.length - 3} more warnings
              </p>
            )}
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
