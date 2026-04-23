'use client'

import { ListTodo } from 'lucide-react'
import type { ActivityItem } from './types'

export function ActivityTodoCard({ event }: { event: ActivityItem }) {
  return (
    <div className="rounded-aurora-2 border border-aurora-border-default bg-aurora-panel-medium px-3 py-3">
      <div className="flex items-start gap-2">
        <ListTodo className="size-4 text-aurora-accent-primary" />
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium text-aurora-text-primary">{event.title ?? 'Plan updated'}</p>
          <div className="mt-2 flex flex-col gap-2">
            {(event.plan ?? []).map((entry, index) => (
              <div key={`${entry.content}-${index}`} className="rounded border border-aurora-border-default/60 bg-aurora-page-bg px-2 py-2 text-xs">
                <div className="flex items-center justify-between gap-2">
                  <span className="font-medium text-aurora-text-primary">{entry.content}</span>
                  <span className="text-aurora-text-muted">{entry.status}</span>
                </div>
                <p className="mt-1 text-[11px] uppercase tracking-[0.14em] text-aurora-text-muted/60">{entry.priority}</p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
