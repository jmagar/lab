'use client'

import { Bug } from 'lucide-react'
import type { ActivityItem } from './types'

export function ActivityDebugCard({ event }: { event: ActivityItem }) {
  return (
    <div className="rounded-aurora-2 border border-aurora-border-default bg-aurora-panel-medium px-3 py-3">
      <div className="flex items-start gap-2">
        <Bug className="size-4 text-aurora-error" />
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium text-aurora-text-primary">{event.title ?? 'Debug event'}</p>
          {event.text ? <p className="mt-1 whitespace-pre-wrap text-xs leading-5 text-aurora-text-muted">{event.text}</p> : null}
          {event.raw ? <pre className="mt-2 overflow-x-auto whitespace-pre-wrap rounded border border-aurora-border-default/60 bg-aurora-page-bg px-2 py-2 font-mono text-[11px] text-aurora-text-muted">{JSON.stringify(event.raw, null, 2)}</pre> : null}
        </div>
      </div>
    </div>
  )
}
