'use client'

import { Activity, BarChart3, Settings2, Sparkles } from 'lucide-react'
import type { ActivityItem } from './types'

function iconFor(kind: ActivityItem['kind']) {
  switch (kind) {
    case 'usage':
      return <BarChart3 className="size-4 text-aurora-accent-primary" />
    case 'mode':
    case 'config':
      return <Settings2 className="size-4 text-aurora-accent-primary" />
    default:
      return <Activity className="size-4 text-aurora-accent-primary" />
  }
}

export function ActivityStatusCard({ event }: { event: ActivityItem }) {
  return (
    <div className="rounded-aurora-2 border border-aurora-border-default bg-aurora-panel-medium px-3 py-3">
      <div className="flex items-start gap-2">
        {iconFor(event.kind)}
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium text-aurora-text-primary">{event.title ?? 'Status update'}</p>
          {event.text ? <p className="mt-1 whitespace-pre-wrap text-xs leading-5 text-aurora-text-muted">{event.text}</p> : null}
          {event.usage ? (
            <div className="mt-2 grid grid-cols-2 gap-2 text-xs text-aurora-text-muted">
              <span>Used: {event.usage.used}</span>
              <span>Window: {event.usage.size}</span>
            </div>
          ) : null}
          {event.sessionInfo?.title ? (
            <div className="mt-2 inline-flex items-center gap-1 rounded-full border border-aurora-border-default bg-aurora-control-surface px-2 py-1 text-[11px] text-aurora-text-primary">
              <Sparkles className="size-3 text-aurora-accent-primary" />
              <span>{event.sessionInfo.title}</span>
            </div>
          ) : null}
          {event.currentMode ? (
            <pre className="mt-2 overflow-x-auto whitespace-pre-wrap rounded border border-aurora-border-default/60 bg-aurora-page-bg px-2 py-2 font-mono text-[11px] text-aurora-text-muted">{JSON.stringify(event.currentMode, null, 2)}</pre>
          ) : null}
          {event.configUpdate ? (
            <pre className="mt-2 overflow-x-auto whitespace-pre-wrap rounded border border-aurora-border-default/60 bg-aurora-page-bg px-2 py-2 font-mono text-[11px] text-aurora-text-muted">{JSON.stringify(event.configUpdate.configOptions, null, 2)}</pre>
          ) : null}
          {event.commands ? (
            <div className="mt-2 flex flex-wrap gap-1.5">
              {event.commands.map((command) => (
                <span key={command.name} className="rounded-full border border-aurora-border-default bg-aurora-control-surface px-2 py-1 text-[11px] text-aurora-text-primary">
                  {command.name}
                </span>
              ))}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  )
}
