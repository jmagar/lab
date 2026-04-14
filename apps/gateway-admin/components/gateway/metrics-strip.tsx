import { Wrench, Eye, Filter } from 'lucide-react'
import { cn } from '@/lib/utils'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'

interface MetricsStripProps {
  discoveredCount: number
  exposedCount: number
  showFiltered?: boolean
  className?: string
}

export function MetricsStrip({ 
  discoveredCount, 
  exposedCount, 
  showFiltered = true,
  className 
}: MetricsStripProps) {
  const filteredCount = discoveredCount - exposedCount

  return (
    <TooltipProvider>
      <div className={cn('flex items-center gap-3 text-sm', className)}>
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="flex items-center gap-1.5 text-muted-foreground">
              <Wrench className="size-3.5" />
              <span className="tabular-nums">{discoveredCount}</span>
            </div>
          </TooltipTrigger>
          <TooltipContent>Discovered MCP tools</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <div className="flex items-center gap-1.5 text-emerald-600 dark:text-emerald-400">
              <Eye className="size-3.5" />
              <span className="tabular-nums">{exposedCount}</span>
            </div>
          </TooltipTrigger>
          <TooltipContent>Exposed downstream</TooltipContent>
        </Tooltip>

        {showFiltered && filteredCount > 0 && (
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-1.5 text-muted-foreground">
                <Filter className="size-3.5" />
                <span className="tabular-nums">{filteredCount}</span>
              </div>
            </TooltipTrigger>
            <TooltipContent>Filtered by allowlist</TooltipContent>
          </Tooltip>
        )}
      </div>
    </TooltipProvider>
  )
}
