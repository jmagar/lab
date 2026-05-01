import type { ReactNode } from 'react'

import { AppHeader } from '@/components/app-header'
import { SettingsRail } from '@/components/settings/SettingsRail'
import { DraftStaleBanner } from '@/components/settings/DraftStaleBanner'

export default function SettingsLayout({
  children,
}: {
  children: ReactNode
}): React.ReactElement {
  return (
    <div className="flex flex-col">
      <AppHeader />
      <div className="grid grid-cols-[220px_1fr] gap-6 p-6">
        <aside className="rounded-md border bg-card">
          <SettingsRail />
        </aside>
        <main className="flex flex-col gap-4">
          <DraftStaleBanner />
          {children}
        </main>
      </div>
    </div>
  )
}
