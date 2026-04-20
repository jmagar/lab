'use client'

import { useState } from 'react'
import { RegistryListContent } from '@/components/registry/registry-list-content'
import { ServerDetailPanel } from '@/components/registry/server-detail-panel'
import type { ServerJSON } from '@/lib/types/registry'

export default function RegistryPage() {
  const [selectedServer, setSelectedServer] = useState<ServerJSON | null>(null)

  return (
    <>
      <RegistryListContent onSelectServer={setSelectedServer} />
      <ServerDetailPanel
        server={selectedServer}
        onClose={() => setSelectedServer(null)}
      />
    </>
  )
}
