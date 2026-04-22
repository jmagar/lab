'use client'

import * as React from 'react'
import { Copy, Check, Brain, ChevronDown, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { ToolCallDisplay } from './tool-call-display'
import type { ACPMessage, ACPToolResultPart, ACPToolUsePart } from './types'

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = React.useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={handleCopy}
      className="size-6 shrink-0 rounded text-aurora-text-muted/40 opacity-0 transition-opacity group-hover/bubble:opacity-100 hover:bg-aurora-hover-bg hover:text-aurora-text-muted"
    >
      {copied ? <Check className="size-3 text-aurora-success" /> : <Copy className="size-3" />}
    </Button>
  )
}

function ThinkingBlock({ thinking }: { thinking: string }) {
  const [open, setOpen] = React.useState(false)

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <CollapsibleTrigger asChild>
        <button
          type="button"
          className="flex items-center gap-1.5 rounded-full border border-aurora-border-default/60 bg-aurora-control-surface/50 px-2.5 py-1 text-[11px] text-aurora-text-muted/70 transition-colors hover:border-aurora-border-strong hover:text-aurora-text-muted"
        >
          <Brain className="size-3 text-aurora-accent-primary/60" />
          <span>Thinking</span>
          {open ? <ChevronDown className="size-3" /> : <ChevronRight className="size-3" />}
        </button>
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className="mt-2 rounded-aurora-1 border border-aurora-border-default/40 bg-aurora-control-surface/30 px-3 py-2.5">
          <p className="font-mono text-[11px] leading-[1.6] text-aurora-text-muted/70 whitespace-pre-wrap">
            {thinking}
          </p>
        </div>
      </CollapsibleContent>
    </Collapsible>
  )
}

function TextContent({ text }: { text: string }) {
  const paragraphs = text.split('\n\n').filter(Boolean)

  return (
    <div className="space-y-2">
      {paragraphs.map((para, i) => {
        if (para.startsWith('# ')) {
          return <p key={i} className="font-display text-[17px] font-bold tracking-[-0.01em] text-aurora-text-primary">{para.slice(2)}</p>
        }
        if (para.startsWith('## ')) {
          return <p key={i} className="font-display text-[15px] font-bold tracking-[-0.01em] text-aurora-text-primary">{para.slice(3)}</p>
        }
        // Inline bold: **text**
        const rendered = para.split(/(\*\*[^*]+\*\*)/).map((chunk, j) => {
          if (chunk.startsWith('**') && chunk.endsWith('**')) {
            return <strong key={j} className="font-semibold text-aurora-text-primary">{chunk.slice(2, -2)}</strong>
          }
          // Handle `code` inline
          return chunk.split(/(`[^`]+`)/).map((c, k) => {
            if (c.startsWith('`') && c.endsWith('`')) {
              return <code key={k} className="rounded px-1 py-0.5 font-mono text-[11px] bg-aurora-control-surface text-aurora-accent-strong">{c.slice(1, -1)}</code>
            }
            return c
          })
        })

        if (para.startsWith('- ') || para.includes('\n- ')) {
          const lines = para.split('\n')
          return (
            <ul key={i} className="space-y-0.5 pl-4">
              {lines.map((line, j) => (
                <li key={j} className="list-disc text-[13px] leading-[1.55] text-aurora-text-primary">
                  {line.replace(/^- /, '')}
                </li>
              ))}
            </ul>
          )
        }

        return (
          <p key={i} className="text-[13px] leading-[1.55] text-aurora-text-primary whitespace-pre-wrap">
            {rendered}
          </p>
        )
      })}
    </div>
  )
}

export function MessageBubble({ message }: { message: ACPMessage }) {
  const isUser = message.role === 'user'
  const textContent = message.parts.filter((p) => p.type === 'text').map((p) => (p as { type: 'text'; text: string }).text).join('\n\n')

  // Pair tool_use parts with their results
  const toolPairs = React.useMemo(() => {
    const uses = message.parts.filter((p): p is ACPToolUsePart => p.type === 'tool_use')
    const results = message.parts.filter((p): p is ACPToolResultPart => p.type === 'tool_result')
    return uses.map((use) => ({
      use,
      result: results.find((r) => r.tool_use_id === use.id),
    }))
  }, [message.parts])

  const thinkingParts = message.parts.filter((p) => p.type === 'thinking')

  return (
    <div className={cn('group/bubble flex gap-3', isUser && 'flex-row-reverse')}>
      {/* Avatar dot */}
      <div className={cn(
        'mt-1 size-6 shrink-0 rounded-full border flex items-center justify-center text-[10px] font-bold',
        isUser
          ? 'border-aurora-border-strong bg-aurora-panel-strong text-aurora-text-muted'
          : 'border-aurora-accent-primary/30 bg-aurora-accent-deep/20 text-aurora-accent-primary',
      )}>
        {isUser ? 'U' : 'A'}
      </div>

      {/* Content */}
      <div className={cn('flex min-w-0 max-w-[80%] flex-col gap-2', isUser && 'items-end')}>
        {/* Thinking block (agent only) */}
        {!isUser && thinkingParts.map((p, i) => (
          <ThinkingBlock key={i} thinking={(p as { type: 'thinking'; thinking: string }).thinking} />
        ))}

        {/* Tool calls (agent only) */}
        {!isUser && toolPairs.length > 0 && (
          <div className="w-full space-y-1.5">
            {toolPairs.map(({ use, result }) => (
              <ToolCallDisplay key={use.id} toolUse={use} toolResult={result} isPending={message.isStreaming && !result} />
            ))}
          </div>
        )}

        {/* Text content */}
        {textContent && (
          <div className={cn(
            'relative rounded-aurora-2 px-4 py-3',
            isUser
              ? 'bg-aurora-panel-strong border border-aurora-border-strong shadow-[var(--aurora-shadow-medium),var(--aurora-highlight-medium)]'
              : 'bg-aurora-panel-medium border border-aurora-border-default shadow-[var(--aurora-shadow-medium),var(--aurora-highlight-medium)]',
          )}>
            {isUser ? (
              <p className="text-[13px] leading-[1.55] text-aurora-text-primary whitespace-pre-wrap">{textContent}</p>
            ) : (
              <TextContent text={textContent} />
            )}

            {/* Copy button */}
            <div className="absolute right-2 top-2">
              <CopyButton text={textContent} />
            </div>

            {/* Streaming cursor */}
            {message.isStreaming && (
              <span className="ml-0.5 inline-block h-3.5 w-0.5 animate-pulse rounded-sm bg-aurora-accent-primary align-middle" />
            )}
          </div>
        )}
      </div>
    </div>
  )
}
