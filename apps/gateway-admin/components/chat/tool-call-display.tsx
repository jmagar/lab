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

function actionLabel(toolCall: TranscriptToolCall) {
  const title = toolCall.title.toLowerCase()
  const kind = toolCall.kind?.toLowerCase() ?? ''

  if (title.includes('skill')) return 'Reading workflow guidance'
  if (title.includes('search') || kind.includes('search')) return toolCall.title
  if (title.includes('photo') || title.includes('image')) return toolCall.title
  if (title.includes('edit') || title.includes('patch') || title.includes('write')) return toolCall.title
  if (title.includes('read') || title.includes('file') || kind.includes('read')) return title.startsWith('read') ? toolCall.title : `Reading ${toolCall.title}`

  return toolCall.title
}

function flattenText(value: unknown): string[] {
  if (typeof value === 'string') {
    const trimmed = value.trim()
    return trimmed ? [trimmed] : []
  }

  if (Array.isArray(value)) {
    return value.flatMap((item) => flattenText(item))
  }

  if (value && typeof value === 'object') {
    return Object.values(value as Record<string, unknown>).flatMap((item) => flattenText(item))
  }

  return []
}

function findUrls(value: unknown): string[] {
  const urls = new Set<string>()
  const urlPattern = /https?:\/\/[^\s"'<>]+/g

  for (const text of flattenText(value)) {
    for (const match of text.matchAll(urlPattern)) {
      urls.add(match[0])
    }
  }

  return [...urls]
}

function toDomainChip(url: string) {
  try {
    return new URL(url).hostname.replace(/^www\./, '')
  } catch {
    return url
  }
}

function getParsedCommands(toolCall: TranscriptToolCall) {
  const input =
    toolCall.input && typeof toolCall.input === 'object'
      ? (toolCall.input as Record<string, unknown>)
      : null
  const parsed =
    input && Array.isArray(input.parsed_cmd)
      ? (input.parsed_cmd as Array<Record<string, unknown>>)
      : []

  return parsed
    .map((entry) => {
      const cmd = typeof entry.cmd === 'string' ? entry.cmd : null
      const path = typeof entry.path === 'string' ? entry.path : null
      const name = typeof entry.name === 'string' ? entry.name : path?.split('/').at(-1) ?? null
      const type = typeof entry.type === 'string' ? entry.type : null

      return { cmd, path, name, type }
    })
    .filter((entry) => entry.cmd || entry.path || entry.name)
}

function getInlineArtifact(toolCall: TranscriptToolCall) {
  const content = Array.isArray(toolCall.content) ? toolCall.content : []
  const inputText = flattenText(toolCall.input)
  const outputText = flattenText(toolCall.output)
  const contentText = flattenText(content)
  const textSnippets = [...inputText, ...contentText, ...outputText]
    .map((text) => text.replace(/\s+/g, ' ').trim())
    .filter((text) => text.length > 0)
    .filter((text, index, values) => values.indexOf(text) === index)

  const urls = [
    ...findUrls(toolCall.input),
    ...findUrls(content),
    ...findUrls(toolCall.output),
  ].filter((url, index, values) => values.indexOf(url) === index)

  const imageUrl =
    urls.find((url) => /\.(png|jpe?g|gif|webp|svg)(\?|$)/i.test(url)) ??
    urls.find((url) => /image|photo|avatar/i.test(url))

  const commands = getParsedCommands(toolCall)
  const summary =
    textSnippets.find((text) => text.length > 48 && !text.startsWith('{') && !text.startsWith('[')) ??
    null

  return {
    commands,
    summary,
    imageUrl: imageUrl ?? null,
    links: urls.filter((url) => url !== imageUrl).slice(0, 4),
  }
}

export function ToolCallDisplay({ toolCall }: ToolCallDisplayProps) {
  const [open, setOpen] = React.useState(false)
  const artifact = React.useMemo(() => getInlineArtifact(toolCall), [toolCall])

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
                {actionLabel(toolCall)}
              </span>
              <span className="mt-0.5 shrink-0">{statusIcon}</span>
              {open ? (
                <ChevronDown className="mt-0.5 size-3.5 shrink-0 text-aurora-text-muted/50" />
              ) : (
                <ChevronRight className="mt-0.5 size-3.5 shrink-0 text-aurora-text-muted/50" />
              )}
            </div>
            <div className="mt-1">
              <span className={cn(AURORA_MUTED_LABEL, 'text-aurora-text-muted/55')}>
                {toolCall.status === 'completed'
                  ? 'completed'
                  : toolCall.status === 'failed'
                    ? 'failed'
                    : 'running'}
              </span>
            </div>
            {toolCall.locations.length > 0 && (
              <div className="mt-2 flex flex-wrap gap-2">
                {toolCall.locations.map((location) => (
                  <span
                    key={location}
                    className="max-w-full truncate rounded-full border border-aurora-border-default bg-aurora-control-surface px-2.5 py-1 text-[11px] text-aurora-text-primary"
                    title={location}
                  >
                    {location}
                  </span>
                ))}
              </div>
            )}
            {artifact.commands.length > 0 && (
              <div className="mt-2 space-y-1.5">
                {artifact.commands.slice(0, 2).map((command, index) => (
                  <div
                    key={`${command.cmd ?? command.path ?? 'command'}-${index}`}
                    className="rounded-aurora-1 border border-aurora-border-default/70 bg-aurora-control-surface px-2.5 py-2"
                  >
                    <p className="text-[12px] font-medium text-aurora-text-primary">
                      {command.type === 'read'
                        ? `Reading ${command.name ?? 'resource'}`
                        : command.type === 'search'
                          ? `Searching ${command.name ?? 'source'}`
                          : command.type === 'write' || command.type === 'edit'
                            ? `Editing ${command.name ?? 'resource'}`
                            : command.name ?? command.path ?? 'Running command'}
                    </p>
                    {command.path ? (
                      <p className="mt-1 truncate text-[11px] leading-[1.45] text-aurora-text-muted">
                        {command.path}
                      </p>
                    ) : null}
                  </div>
                ))}
              </div>
            )}
            {artifact.links.length > 0 && (
              <div className="mt-2 flex flex-wrap gap-2">
                {artifact.links.map((url) => (
                  <a
                    key={url}
                    href={url}
                    target="_blank"
                    rel="noreferrer"
                    className="rounded-full border border-aurora-border-default bg-aurora-control-surface px-2.5 py-1 text-[11px] text-aurora-text-primary transition-colors hover:border-aurora-accent-primary/50 hover:text-aurora-accent-primary"
                  >
                    {toDomainChip(url)}
                  </a>
                ))}
              </div>
            )}
            {artifact.imageUrl ? (
              <div className="mt-2 overflow-hidden rounded-aurora-2 border border-aurora-border-default bg-aurora-control-surface">
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img
                  src={artifact.imageUrl}
                  alt={toolCall.title}
                  className="max-h-48 w-full object-cover"
                />
              </div>
            ) : null}
            {artifact.summary ? (
              <p className="mt-2 text-[12px] leading-[1.55] text-aurora-text-muted">
                {artifact.summary}
              </p>
            ) : null}
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
