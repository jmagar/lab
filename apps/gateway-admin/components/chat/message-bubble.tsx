'use client'

import * as React from 'react'
import { Bot, Copy, Check, UserRound } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import {
  ChainOfThought,
  ChainOfThoughtContent,
  ChainOfThoughtHeader,
} from '@/components/ai/chain-of-thought'
import { Reasoning, ReasoningContent, ReasoningTrigger } from '@/components/ai/reasoning'
import { ToolCallDisplay } from './tool-call-display'
import type { ACPMessage } from './types'
import { AURORA_MUTED_LABEL } from '@/components/aurora/tokens'

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = React.useState(false)

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text)
      setCopied(true)
      window.setTimeout(() => setCopied(false), 2000)
    } catch {
      setCopied(false)
    }
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

function StreamingCursor() {
  return (
    <span
      aria-hidden="true"
      className="ml-0.5 inline-block h-3.5 w-0.5 animate-pulse rounded-sm bg-aurora-accent-primary align-middle"
    />
  )
}

export function MessageBubble({ message }: { message: ACPMessage }) {
  const isUser = message.role === 'user'
  const [reasoningOpen, setReasoningOpen] = React.useState(Boolean(message.isStreaming))
  const [chainOpen, setChainOpen] = React.useState(Boolean(message.isStreaming))

  React.useEffect(() => {
    const streaming = Boolean(message.isStreaming)
    setReasoningOpen(streaming)
    setChainOpen(streaming || message.toolCalls.length > 0)
  }, [message.isStreaming, message.toolCalls.length])

  const hasChain = !isUser && (message.thoughts.length > 0 || message.toolCalls.length > 0)

  return (
    <div className={cn('group/bubble flex gap-3', isUser && 'flex-row-reverse')}>
      <div
        className={cn(
          'mt-1 flex size-6 shrink-0 items-center justify-center rounded-full border',
          isUser
            ? 'border-aurora-border-strong bg-aurora-panel-strong'
            : 'border-aurora-accent-primary/30 bg-aurora-accent-deep/18',
        )}
      >
        {isUser ? (
          <UserRound className="size-3 text-aurora-text-muted" />
        ) : (
          <Bot className="size-3 text-aurora-accent-primary" />
        )}
      </div>

      <div className={cn('flex min-w-0 max-w-[92%] flex-col gap-2.5 sm:max-w-[80%]', isUser && 'items-end')}>
        {hasChain && (
          <div className="w-full overflow-hidden rounded-aurora-3 border border-aurora-border-default bg-aurora-panel-medium shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong)]">
            <ChainOfThought
              open={chainOpen}
              onOpenChange={setChainOpen}
              className="px-4 py-3"
            >
              <ChainOfThoughtHeader className="font-medium text-aurora-text-muted">
                Chain of Thought
              </ChainOfThoughtHeader>
              <ChainOfThoughtContent className="pt-1">
                {message.thoughts.length > 0 && (
                  <div className="rounded-aurora-2 border border-aurora-border-default/70 bg-aurora-control-surface px-3 py-3">
                    <Reasoning
                      isStreaming={Boolean(message.isStreaming)}
                      open={reasoningOpen}
                      onOpenChange={setReasoningOpen}
                      className="mb-0"
                    >
                      <ReasoningTrigger className="text-aurora-text-muted" />
                      <ReasoningContent className="mt-3 space-y-3 text-aurora-text-primary">
                        {message.thoughts.join('\n\n')}
                      </ReasoningContent>
                    </Reasoning>
                  </div>
                )}

                {message.toolCalls.length > 0 && (
                  <div className="space-y-1.5 pt-1">
                    <div className="flex items-center gap-2 px-1">
                      <span className={cn(AURORA_MUTED_LABEL, 'text-aurora-text-muted/60')}>
                        agent actions
                      </span>
                    </div>
                    <div className="space-y-1">
                      {message.toolCalls.map((toolCall) => (
                        <ToolCallDisplay key={toolCall.id} toolCall={toolCall} />
                      ))}
                    </div>
                  </div>
                )}
              </ChainOfThoughtContent>
            </ChainOfThought>
          </div>
        )}

        {message.text && (
          <div
            className={cn(
              'relative overflow-hidden rounded-aurora-2 px-4 py-3',
              isUser
                ? 'border border-aurora-border-strong bg-aurora-panel-strong shadow-[var(--aurora-shadow-medium),var(--aurora-highlight-medium)]'
                : 'border border-aurora-border-default bg-aurora-panel-medium shadow-[var(--aurora-shadow-medium),var(--aurora-highlight-medium)]',
            )}
          >
            {!isUser && (
              <span
                aria-hidden="true"
                className="absolute inset-y-0 left-0 w-[2px] rounded-l-aurora-2 bg-aurora-accent-primary/40"
              />
            )}
            <p className="whitespace-pre-wrap pr-8 text-[13px] leading-[1.55] text-aurora-text-primary">
              {message.text}
              {message.isStreaming ? <StreamingCursor /> : null}
            </p>
            <div className="absolute right-2 top-2">
              <CopyButton text={message.text} />
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
