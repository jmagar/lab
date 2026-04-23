'use client'

import { ShieldCheck } from 'lucide-react'
import type { ActivityItem } from './types'

export function ActivityPermissionCard({ event }: { event: ActivityItem }) {
  return (
    <div className="rounded-aurora-2 border border-aurora-border-default bg-aurora-panel-medium px-3 py-3">
      <div className="flex items-start gap-2">
        <ShieldCheck className="size-4 text-aurora-accent-primary" />
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium text-aurora-text-primary">{event.title ?? 'Permission request'}</p>
          {event.permissionOptions?.length ? (
            <div className="mt-2 flex flex-wrap gap-1.5">
              {event.permissionOptions.map((option) => (
                <span key={option.optionId} className="rounded-full border border-aurora-border-default bg-aurora-control-surface px-2 py-1 text-[11px] text-aurora-text-primary">
                  {option.name}
                </span>
              ))}
            </div>
          ) : null}
          {event.permissionSelection ? (
            <p className="mt-2 text-xs text-aurora-text-muted">Resolved with: {event.permissionSelection}</p>
          ) : null}
        </div>
      </div>
    </div>
  )
}
