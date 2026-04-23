'use client'

import * as React from 'react'
import { ChevronDown, ChevronRight, Terminal, CheckCircle2, XCircle, Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import type { TranscriptToolCall } from './types'

interface ToolCallDisplayProps {
  toolCall: TranscriptToolCall
}

export function ToolCallDisplay({ toolCall }: ToolCallDisplayProps) {
  const [open, setOpen] = React.useState(false)

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
            'group flex w-full items-center gap-2 rounded-aurora-1 border border-aurora-border-default',
            'bg-aurora-control-surface px-3 py-2 text-left transition-colors hover:border-aurora-border-strong',
          )}
        >
          <Terminal className="size-3.5 shrink-0 text-aurora-accent-primary/70" />
          <span className="flex-1 text-[12px] font-medium text-aurora-text-primary">
            {toolCall.title}
          </span>
          {statusIcon}
          {open ? (
            <ChevronDown className="size-3 text-aurora-text-muted/50" />
          ) : (
            <ChevronRight className="size-3 text-aurora-text-muted/50" />
          )}
        </button>
      </CollapsibleTrigger>

      <CollapsibleContent>
        <div className="mt-1 overflow-hidden rounded-b-aurora-1 rounded-t-sm border border-t-0 border-aurora-border-default bg-aurora-page-bg">
          <div className="border-b border-aurora-border-default/60 px-3 py-2">
            <p className="mb-1.5 text-[10px] font-bold uppercase tracking-[0.14em] text-aurora-text-muted/60">
              Input
            </p>
            <pre className="overflow-x-auto font-mono text-[11px] leading-[1.5] text-aurora-text-primary">
              {JSON.stringify(toolCall.input ?? {}, null, 2)}
            </pre>
          </div>

          {toolCall.output !== undefined && (
            <div className="px-3 py-2">
              <p className="mb-1.5 text-[10px] font-bold uppercase tracking-[0.14em] text-aurora-success/70">
                Output
              </p>
              <pre className="overflow-x-auto font-mono text-[11px] leading-[1.5] whitespace-pre-wrap text-aurora-text-primary">
                {typeof toolCall.output === 'string'
                  ? toolCall.output
                  : JSON.stringify(toolCall.output, null, 2)}
              </pre>
            </div>
          )}
        </div>
      </CollapsibleContent>
    </Collapsible>
  )
}
