'use client'

import * as React from 'react'
import {
  BookOpen,
  ChevronDown,
  ChevronRight,
  CheckCircle2,
  XCircle,
  FileText,
  Loader2,
  Pencil,
  Search,
  Terminal,
  Wrench,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { ToolCallDisplay } from './tool-call-display'
import type { TranscriptToolCall } from './types'

interface GroupedToolCallDisplayProps {
  category: string
  toolCalls: TranscriptToolCall[]
}

const CATEGORY_ICON: Record<string, React.ElementType> = {
  skill: BookOpen,
  read: FileText,
  edit: Pencil,
  search: Search,
  command: Terminal,
  tool: Wrench,
}

const CATEGORY_COLOR: Record<string, string> = {
  skill: 'text-sky-400',
  read: 'text-blue-400',
  edit: 'text-orange-400',
  search: 'text-emerald-400',
  command: 'text-lime-400',
  tool: 'text-aurora-accent-primary',
}

function groupLabel(category: string, n: number): string {
  if (category === 'read') return `Read ${n} ${n === 1 ? 'file' : 'files'}`
  if (category === 'edit') return `Edited ${n} ${n === 1 ? 'file' : 'files'}`
  if (category === 'search') return `Searched ${n} ${n === 1 ? 'directory' : 'directories'}`
  if (category === 'command') return `Ran ${n} ${n === 1 ? 'command' : 'commands'}`
  if (category === 'skill') return `Loaded ${n} ${n === 1 ? 'skill' : 'skills'}`
  return `${n} actions`
}

function chipLabel(toolCall: TranscriptToolCall): string {
  // Strip verb prefix, fall back to last path segment or title
  const stripped = toolCall.title.replace(/^(Reading|Editing|Searching|Running)\s+/i, '')
  if (stripped) return stripped
  const loc = toolCall.locations[0]
  return loc ? (loc.split('/').at(-1) ?? loc) : toolCall.title
}

function groupStatus(toolCalls: TranscriptToolCall[]): 'completed' | 'failed' | 'running' {
  if (toolCalls.some((t) => t.status === 'failed')) return 'failed'
  if (toolCalls.every((t) => t.status === 'completed')) return 'completed'
  return 'running'
}

export function GroupedToolCallDisplay({ category, toolCalls }: GroupedToolCallDisplayProps) {
  const [open, setOpen] = React.useState(false)

  const Icon = CATEGORY_ICON[category] ?? Wrench
  const colorClass = CATEGORY_COLOR[category] ?? 'text-aurora-accent-primary'
  const status = groupStatus(toolCalls)
  const label = groupLabel(category, toolCalls.length)

  const statusIcon =
    status === 'failed' ? (
      <XCircle className="size-3.5 text-aurora-error" />
    ) : status === 'completed' ? (
      <CheckCircle2 className="size-3.5 text-aurora-success" />
    ) : (
      <Loader2 className="size-3.5 animate-spin text-aurora-accent-primary" />
    )

  const chips = toolCalls.slice(0, 4).map((tc) => chipLabel(tc))
  const overflow = toolCalls.length - 4

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
            <Icon className={cn('size-4 shrink-0', colorClass)} />
          </span>
          <div className="min-w-0 flex-1 rounded-aurora-1 px-1 py-0.5 transition-colors group-hover:bg-aurora-hover-bg/30">
            <div className="flex min-w-0 items-start gap-2">
              <div className="min-w-0 flex-1">
                <span className="text-[14px] leading-[1.45] text-aurora-text-primary">
                  {label}
                </span>
                <span className="ml-2 inline-flex items-center rounded-full border border-aurora-border-default bg-aurora-control-surface px-2 py-0.5 text-[10px] font-semibold text-aurora-text-muted">
                  {toolCalls.length}
                </span>
                {!open && (
                  <div className="mt-1.5 flex flex-wrap gap-1.5">
                    {chips.map((chip, i) => (
                      <span
                        key={i}
                        className="rounded border border-aurora-border-default bg-aurora-control-surface px-1.5 py-0.5 text-[10px] text-aurora-text-muted"
                      >
                        {chip}
                      </span>
                    ))}
                    {overflow > 0 && (
                      <span className="rounded border border-aurora-border-default bg-aurora-control-surface px-1.5 py-0.5 text-[10px] text-aurora-text-muted">
                        +{overflow}
                      </span>
                    )}
                  </div>
                )}
              </div>
              <span className="mt-0.5 shrink-0">{statusIcon}</span>
              {open ? (
                <ChevronDown className="mt-0.5 size-3.5 shrink-0 text-aurora-text-muted/50" />
              ) : (
                <ChevronRight className="mt-0.5 size-3.5 shrink-0 text-aurora-text-muted/50" />
              )}
            </div>
          </div>
        </button>
      </CollapsibleTrigger>

      <CollapsibleContent>
        <div className="ml-7 space-y-0.5 border-l border-aurora-border-default/50 pl-3 pt-0.5 pb-1">
          {toolCalls.map((toolCall) => (
            <ToolCallDisplay key={toolCall.id} toolCall={toolCall} isChild />
          ))}
        </div>
      </CollapsibleContent>
    </Collapsible>
  )
}

// Categories that are semantically heterogeneous — never group them.
const UNGROUPABLE = new Set(['tool', 'permission', 'plan', 'review', 'media', 'source'])

export function groupConsecutiveToolCalls(
  toolCalls: TranscriptToolCall[],
  getCategory: (tc: TranscriptToolCall) => string,
  minGroupSize = 2,
): Array<
  | { type: 'single'; toolCall: TranscriptToolCall }
  | { type: 'group'; category: string; toolCalls: TranscriptToolCall[] }
> {
  const result: Array<
    | { type: 'single'; toolCall: TranscriptToolCall }
    | { type: 'group'; category: string; toolCalls: TranscriptToolCall[] }
  > = []

  let i = 0
  while (i < toolCalls.length) {
    const cat = getCategory(toolCalls[i]!)
    let j = i + 1
    while (j < toolCalls.length && getCategory(toolCalls[j]!) === cat) j++
    const run = toolCalls.slice(i, j)
    if (run.length >= minGroupSize && !UNGROUPABLE.has(cat)) {
      result.push({ type: 'group', category: cat, toolCalls: run })
    } else {
      run.forEach((tc) => result.push({ type: 'single', toolCall: tc }))
    }
    i = j
  }

  return result
}
