'use client'

import { Brain, Wrench } from 'lucide-react'
import { ActivityDebugCard } from './activity-debug-card'
import { ActivityPermissionCard } from './activity-permission-card'
import { ActivityReviewCard } from './activity-review-card'
import { ActivityStatusCard } from './activity-status-card'
import { ActivityTodoCard } from './activity-todo-card'
import type { ActivityItem } from './types'

function ThoughtCard({ event }: { event: ActivityItem }) {
  return (
    <div className="rounded-aurora-2 border border-aurora-border-default bg-aurora-panel-medium px-3 py-3">
      <div className="flex items-start gap-2">
        <Brain className="size-4 text-aurora-accent-primary" />
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium text-aurora-text-primary">Reasoning update</p>
          <p className="mt-1 whitespace-pre-wrap text-xs leading-5 text-aurora-text-muted">{event.text}</p>
        </div>
      </div>
    </div>
  )
}

function ToolCard({ event }: { event: ActivityItem }) {
  return (
    <div className="rounded-aurora-2 border border-aurora-border-default bg-aurora-panel-medium px-3 py-3">
      <div className="flex items-start gap-2">
        <Wrench className="size-4 text-aurora-accent-primary" />
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium text-aurora-text-primary">{event.title ?? 'Tool activity'}</p>
          <p className="mt-1 text-xs text-aurora-text-muted">Status: {event.status ?? 'unknown'}</p>
          {event.locations?.length ? (
            <div className="mt-2 flex flex-wrap gap-1.5">
              {event.locations.map((location) => (
                <span key={location} className="rounded-full border border-aurora-border-default bg-aurora-control-surface px-2 py-1 text-[11px] text-aurora-text-primary">
                  {location}
                </span>
              ))}
            </div>
          ) : null}
          {event.toolContent?.some((item) => item.type === 'diff') ? (
            <ActivityReviewCard event={event} />
          ) : null}
        </div>
      </div>
    </div>
  )
}

export function ActivityCard({ event }: { event: ActivityItem }) {
  if (event.kind === 'message.chunk' && event.role === 'thinking') {
    return <ThoughtCard event={event} />
  }

  if (event.kind === 'tool.call' || event.kind === 'tool.update') {
    return <ToolCard event={event} />
  }

  if (event.kind === 'permission.requested' || event.kind === 'permission.resolved') {
    return <ActivityPermissionCard event={event} />
  }

  if (event.kind === 'plan') {
    return <ActivityTodoCard event={event} />
  }

  if (event.kind === 'error' || event.kind === 'debug') {
    return <ActivityDebugCard event={event} />
  }

  return <ActivityStatusCard event={event} />
}
