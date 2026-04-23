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
    <div className="flex flex-1 flex-col items-center justify-center gap-3 text-center">
      <div className="flex size-12 items-center justify-center rounded-aurora-2 border border-aurora-border-default bg-aurora-panel-medium">
        <MessageSquare className="size-5 text-aurora-text-muted/50" />
      </div>
      <div className="space-y-1">
        <p className="text-[14px] font-medium text-aurora-text-primary">No session selected</p>
        <p className="text-[13px] text-aurora-text-muted">Select a session from the sidebar or start a new one</p>
      </div>
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
        {messages.map((message) => (
          <MessageBubble key={message.id} message={message} />
        ))}
        <div ref={bottomRef} />
      </div>
    </ScrollArea>
  )
}
