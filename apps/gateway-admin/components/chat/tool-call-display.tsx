'use client'

import * as React from 'react'
import { ChevronDown, ChevronRight, Terminal, CheckCircle2, XCircle, Loader2, CircleDashed } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import type { ACPToolUsePart, ACPToolResultPart } from './types'

interface ToolCallDisplayProps {
  toolUse: ACPToolUsePart
  toolResult?: ACPToolResultPart
  isPending?: boolean
}

export function ToolCallDisplay({ toolUse, toolResult, isPending = false }: ToolCallDisplayProps) {
  const [open, setOpen] = React.useState(false)

  const statusIcon = isPending ? (
    <Loader2 className="size-3.5 animate-spin text-aurora-accent-primary" />
  ) : toolResult?.is_error ? (
    <XCircle className="size-3.5 text-aurora-error" />
  ) : toolResult ? (
    <CheckCircle2 className="size-3.5 text-aurora-success" />
  ) : (
    <CircleDashed className="size-3.5 text-aurora-text-muted/60" />
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
            {toolUse.name}
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
          {/* Input */}
          <div className="border-b border-aurora-border-default/60 px-3 py-2">
            <p className="mb-1.5 text-[10px] font-bold uppercase tracking-[0.14em] text-aurora-text-muted/60">
              Input
            </p>
            <pre className="overflow-x-auto font-mono text-[11px] leading-[1.5] text-aurora-text-primary">
              {JSON.stringify(toolUse.input, null, 2)}
            </pre>
          </div>

          {/* Output */}
          {toolResult && (
            <div className="px-3 py-2">
              <p className={cn(
                'mb-1.5 text-[10px] font-bold uppercase tracking-[0.14em]',
                toolResult.is_error ? 'text-aurora-error/70' : 'text-aurora-success/70',
              )}>
                {toolResult.is_error ? 'Error' : 'Output'}
              </p>
              <pre className={cn(
                'overflow-x-auto font-mono text-[11px] leading-[1.5] whitespace-pre-wrap',
                toolResult.is_error ? 'text-aurora-error' : 'text-aurora-text-primary',
              )}>
                {toolResult.content}
              </pre>
            </div>
          )}

          {isPending && (
            <div className="flex items-center gap-2 px-3 py-2 text-[12px] text-aurora-text-muted/60">
              <Loader2 className="size-3 animate-spin" />
              Running…
            </div>
          )}
        </div>
      </CollapsibleContent>
    </Collapsible>
  )
}
