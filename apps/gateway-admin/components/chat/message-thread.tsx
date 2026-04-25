'use client'

import * as React from 'react'
import { MessageSquare } from 'lucide-react'
import { ScrollArea } from '@/components/ui/scroll-area'
import { MessageBubble } from './message-bubble'
import type { ACPMessage, ACPRun } from './types'

interface MessageThreadProps {
  run: ACPRun | null
  messages: ACPMessage[]
}

function EmptyState() {
  return (
    <div className="flex flex-1 items-center justify-center px-4 py-8 sm:px-6 sm:py-10">
      <div className="w-full max-w-sm rounded-aurora-2 border border-aurora-border-strong bg-aurora-panel-medium p-5 text-center shadow-[var(--aurora-shadow-medium),var(--aurora-highlight-medium)]">
        <div className="mx-auto flex size-12 items-center justify-center rounded-aurora-2 border border-aurora-border-default bg-aurora-panel-strong">
          <MessageSquare className="size-5 text-aurora-text-muted/50" />
        </div>
        <div className="mt-4 space-y-1">
          <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-aurora-text-muted">
            Conversation
          </p>
          <p className="text-[15px] font-medium text-aurora-text-primary">No session selected</p>
          <p className="text-[13px] leading-[1.55] text-aurora-text-muted">
            Open the sessions drawer or start a new run to begin a chat with the ACP bridge.
          </p>
        </div>
      </div>
    </div>
  )
}

function SessionStatusNotice({ run }: { run: ACPRun }) {
  if (run.status !== 'running' && run.status !== 'waiting_for_permission') {
    return null
  }

  const waitingForPermission = run.status === 'waiting_for_permission'

  return (
    <div className="rounded-aurora-2 border border-aurora-accent-primary/25 bg-aurora-accent-deep/10 px-3 py-2 text-[12px] text-aurora-text-muted shadow-[var(--aurora-highlight-soft)]">
      <div className="flex items-center gap-2">
        <span className="size-1.5 rounded-full bg-aurora-accent-primary animate-pulse" />
        <span className="font-medium text-aurora-text-primary">
          {waitingForPermission ? 'Session waiting for permission' : 'Session still running'}
        </span>
      </div>
      <p className="mt-1 leading-[1.45]">
        {waitingForPermission
          ? 'The ACP bridge is waiting on a permission decision before the turn can continue.'
          : 'The ACP bridge has not received a terminal turn event yet.'}
      </p>
    </div>
  )
}

export function MessageThread({ run, messages }: MessageThreadProps) {
  const bottomRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  if (!run) {
    return <EmptyState />
  }

  return (
    <ScrollArea className="min-h-0 flex-1 overflow-hidden">
      <div className="mx-auto flex w-full max-w-[860px] flex-col gap-4 px-4 py-4 sm:gap-5 sm:px-6 sm:py-6">
        <SessionStatusNotice run={run} />
        {messages.map((message) => (
          <MessageBubble key={message.id} message={message} />
        ))}
        <div ref={bottomRef} />
      </div>
    </ScrollArea>
  )
}
