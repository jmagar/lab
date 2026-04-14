'use client'

import { Settings } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { EmptyState } from '@/components/gateway/empty-state'

export default function SettingsPage() {
  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Settings' }
        ]}
      />
      <div className="flex-1 p-6">
        <div className="rounded-lg border bg-card">
          <EmptyState
            title="Settings coming soon"
            description="This page will contain global configuration options for the MCP gateway control plane."
            icon={<Settings className="size-6 text-muted-foreground" />}
          />
        </div>
      </div>
    </>
  )
}
