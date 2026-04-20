'use client'

import { useState } from 'react'
import { RegistryListContent } from '@/components/registry/registry-list-content'
import { ServerDetailPanel } from '@/components/registry/server-detail-panel'
import type { ServerResponse } from '@/lib/types/registry'

export default function RegistryPage() {
  const [selectedResponse, setSelectedResponse] = useState<ServerResponse | null>(null)

  const extensions = selectedResponse?._meta?.['io.modelcontextprotocol.registry/official']

  return (
    <>
      <RegistryListContent onSelectServer={setSelectedResponse} />
      <ServerDetailPanel
        server={selectedResponse?.server ?? null}
        updatedAt={extensions?.updatedAt}
        status={extensions?.status}
        statusMessage={extensions?.statusMessage}
        onClose={() => setSelectedResponse(null)}
      />
    </>
  )
}
