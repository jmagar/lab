'use client'

import { FileDiff } from 'lucide-react'
import type { ActivityItem } from './types'

export function ActivityReviewCard({ event }: { event: ActivityItem }) {
  return (
    <div className="rounded-aurora-2 border border-aurora-border-default bg-aurora-panel-medium px-3 py-3">
      <div className="flex items-start gap-2">
        <FileDiff className="size-4 text-aurora-accent-primary" />
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium text-aurora-text-primary">{event.title ?? 'Review / edit activity'}</p>
          <pre className="aurora-scrollbar mt-2 overflow-x-auto whitespace-pre-wrap rounded border border-aurora-border-default/60 bg-aurora-page-bg px-2 py-2 font-mono text-[11px] text-aurora-text-muted">{JSON.stringify(event.toolContent, null, 2)}</pre>
        </div>
      </div>
    </div>
  )
}
