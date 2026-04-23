'use client'

import * as React from 'react'
import {
  ChevronDown,
  ChevronRight,
  CheckCircle2,
  XCircle,
  Loader2,
  Search,
  Wrench,
  FileText,
  Pencil,
  Image as ImageIcon,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import type { TranscriptToolCall } from './types'
import { AURORA_MUTED_LABEL } from '@/components/aurora/tokens'

interface ToolCallDisplayProps {
  toolCall: TranscriptToolCall
}

export function ToolCallDisplay({ toolCall }: ToolCallDisplayProps) {
  const [open, setOpen] = React.useState(false)

  const activityIcon = (() => {
    const title = toolCall.title.toLowerCase()
    const kind = toolCall.kind?.toLowerCase() ?? ''
    if (title.includes('search') || kind.includes('search')) {
      return Search
    }
    if (title.includes('image') || title.includes('photo') || kind.includes('image')) {
      return ImageIcon
    }
    if (title.includes('edit') || title.includes('write') || title.includes('patch')) {
      return Pencil
    }
    if (title.includes('read') || title.includes('file') || kind.includes('read')) {
      return FileText
    }
    return Wrench
  })()

  const statusTone =
    toolCall.status === 'failed'
      ? 'text-aurora-error'
      : toolCall.status === 'completed'
        ? 'text-aurora-success'
        : 'text-aurora-accent-primary'

  const statusIcon = toolCall.status === 'failed' ? (
    <XCircle className="size-3.5 text-aurora-error" />
  ) : toolCall.status === 'completed' ? (
    <CheckCircle2 className="size-3.5 text-aurora-success" />
  ) : (
    <Loader2 className="size-3.5 animate-spin text-aurora-accent-primary" />
  )

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <CollapsibleTrigger asChild>
        <button
          type="button"
          className={cn(
            'group relative flex w-full items-start gap-3 py-2 text-left',
            'before:absolute before:bottom-0 before:left-[8px] before:top-0 before:w-px before:bg-aurora-border-default/70',
          )}
        >
          <span className="relative z-10 mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full bg-aurora-page-bg">
            {React.createElement(activityIcon, {
              className: cn('size-4 shrink-0', statusTone),
            })}
          </span>
          <div className="min-w-0 flex-1 rounded-aurora-1 px-1 py-0.5 transition-colors group-hover:bg-aurora-hover-bg/30">
            <div className="flex min-w-0 items-start gap-2">
              <span className="flex-1 text-[14px] leading-[1.45] text-aurora-text-primary">
                {toolCall.title}
              </span>
              <span className="mt-0.5 shrink-0">{statusIcon}</span>
              {open ? (
                <ChevronDown className="mt-0.5 size-3.5 shrink-0 text-aurora-text-muted/50" />
              ) : (
                <ChevronRight className="mt-0.5 size-3.5 shrink-0 text-aurora-text-muted/50" />
              )}
            </div>
            {toolCall.locations.length > 0 && (
              <div className="mt-2 flex flex-wrap gap-2">
                {toolCall.locations.map((location) => (
                  <span
                    key={location}
                    className="rounded-full border border-aurora-border-default bg-aurora-control-surface px-2.5 py-1 text-[11px] text-aurora-text-primary"
                  >
                    {location}
                  </span>
                ))}
              </div>
            )}
          </div>
        </button>
      </CollapsibleTrigger>

      <CollapsibleContent>
        <div className="relative ml-2 mt-1 border-l border-aurora-border-default/70 pl-5">
          <div className="overflow-hidden rounded-aurora-1 border border-aurora-border-default bg-aurora-control-surface">
            <div className="border-b border-aurora-border-default/60 px-3 py-2">
              <p className={cn(AURORA_MUTED_LABEL, 'mb-1.5 text-aurora-text-muted/60')}>
                Input
              </p>
              <pre className="aurora-scrollbar overflow-x-auto font-mono text-[11px] leading-[1.5] text-aurora-text-primary">
                {JSON.stringify(toolCall.input ?? {}, null, 2)}
              </pre>
            </div>

            {toolCall.output !== undefined && (
              <div className="px-3 py-2">
                <p className={cn(AURORA_MUTED_LABEL, 'mb-1.5 text-aurora-success/70')}>
                  Output
                </p>
                <pre className="aurora-scrollbar overflow-x-auto whitespace-pre-wrap font-mono text-[11px] leading-[1.5] text-aurora-text-primary">
                  {typeof toolCall.output === 'string'
                    ? toolCall.output
                    : JSON.stringify(toolCall.output, null, 2)}
                </pre>
              </div>
            )}
          </div>
        </div>
      </CollapsibleContent>
    </Collapsible>
  )
}
