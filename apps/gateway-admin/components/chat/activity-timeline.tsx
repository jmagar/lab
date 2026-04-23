'use client'

import { Search } from 'lucide-react'
import { ActivityGroup } from './activity-group'
import { ActivityCard } from './activity-card'
import type { ActivityItem } from './types'

export function ActivityTimeline({
  events,
  connectionState,
}: {
  events: ActivityItem[]
  connectionState: 'idle' | 'connecting' | 'open' | 'error'
}) {
  if (events.length === 0 && connectionState === 'idle') {
    return null
  }

  return (
    <div className="shrink-0 border-b border-aurora-border-default bg-aurora-nav-bg px-4 py-3">
      <div className="mx-auto flex max-w-[820px] flex-col gap-2">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 text-sm text-aurora-text-primary">
            <Search className="size-4 text-aurora-accent-primary" />
            <span className="font-medium">Reasoning & Activity</span>
          </div>
          <span className="text-[11px] text-aurora-text-muted">{connectionState}</span>
        </div>
        <ActivityGroup title="Live session activity" defaultOpen>
          {events.length === 0 ? (
            <div className="rounded-aurora-2 border border-dashed border-aurora-border-default px-3 py-4 text-sm text-aurora-text-muted">
              Waiting for ACP events...
            </div>
          ) : (
            events.map((event) => <ActivityCard key={event.id} event={event} />)
          )}
        </ActivityGroup>
      </div>
    </div>
  )
}
