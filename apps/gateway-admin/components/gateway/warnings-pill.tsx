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
              'inline-flex items-center gap-1 rounded-full border border-warning/20 bg-warning/10 px-2 py-0.5 text-xs font-medium text-warning',
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
              <p className="text-xs text-muted-foreground">
                +{warnings.length - 3} more warnings
              </p>
            )}
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
