'use client'

import * as React from 'react'
import { Copy, Check, Brain } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { ToolCallDisplay } from './tool-call-display'
import type { ACPMessage } from './types'

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = React.useState(false)

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text)
      setCopied(true)
      window.setTimeout(() => setCopied(false), 2000)
    } catch {}
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={handleCopy}
      aria-label="Copy message"
      className="size-6 shrink-0 rounded text-aurora-text-muted/40 opacity-0 transition-opacity group-hover/bubble:opacity-100 hover:bg-aurora-hover-bg hover:text-aurora-text-muted"
    >
      {copied ? <Check className="size-3 text-aurora-success" /> : <Copy className="size-3" />}
    </Button>
  )
}

export function MessageBubble({ message }: { message: ACPMessage }) {
  const isUser = message.role === 'user'

  return (
    <div className={cn('group/bubble flex gap-3', isUser && 'flex-row-reverse')}>
      <div
        className={cn(
          'mt-1 flex size-6 shrink-0 items-center justify-center rounded-full border text-[10px] font-bold',
          isUser
            ? 'border-aurora-border-strong bg-aurora-panel-strong text-aurora-text-muted'
            : 'border-aurora-accent-primary/30 bg-aurora-accent-deep/20 text-aurora-accent-primary',
        )}
      >
        {isUser ? 'U' : 'A'}
      </div>

      <div className={cn('flex min-w-0 max-w-[80%] flex-col gap-2', isUser && 'items-end')}>
        {!isUser && message.thoughts.length > 0 && (
          <div className="flex items-center gap-1.5 rounded-full border border-aurora-border-default/60 bg-aurora-control-surface/50 px-2.5 py-1 text-[11px] text-aurora-text-muted/70">
            <Brain className="size-3 text-aurora-accent-primary/60" />
            <span>{message.thoughts.length} reasoning update{message.thoughts.length === 1 ? '' : 's'}</span>
          </div>
        )}

        {message.toolCalls.length > 0 && (
          <div className="w-full space-y-1.5">
            {message.toolCalls.map((toolCall) => (
              <ToolCallDisplay key={toolCall.id} toolCall={toolCall} />
            ))}
          </div>
        )}

        {message.text && (
          <div
            className={cn(
              'relative rounded-aurora-2 px-4 py-3',
              isUser
                ? 'border border-aurora-border-strong bg-aurora-panel-strong shadow-[var(--aurora-shadow-medium),var(--aurora-highlight-medium)]'
                : 'border border-aurora-border-default bg-aurora-panel-medium shadow-[var(--aurora-shadow-medium),var(--aurora-highlight-medium)]',
            )}
          >
            <p className="whitespace-pre-wrap text-[13px] leading-[1.55] text-aurora-text-primary">{message.text}</p>
            <div className="absolute right-2 top-2">
              <CopyButton text={message.text} />
            </div>
            {message.isStreaming && (
              <span className="ml-0.5 inline-block h-3.5 w-0.5 animate-pulse rounded-sm bg-aurora-accent-primary align-middle" />
            )}
          </div>
        )}
      </div>
    </div>
  )
}
