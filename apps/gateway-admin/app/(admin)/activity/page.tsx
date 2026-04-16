'use client'

import { Activity } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { EmptyState } from '@/components/gateway/empty-state'

export default function ActivityPage() {
  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Activity' }
        ]}
      />
      <div className="flex-1 p-6">
        <div className="rounded-lg border bg-card">
          <EmptyState
            title="Activity feed coming soon"
            description="This page will display gateway events, connection logs, and tool invocation history."
            icon={<Activity className="size-6 text-muted-foreground" />}
          />
        </div>
      </div>
    </>
  )
}
