'use client'

import { useState } from 'react'
import { RegistryListContent } from '@/components/registry/registry-list-content'
import type { ServerJSON } from '@/lib/types/registry'

export default function RegistryPage() {
  const [selectedServer, setSelectedServer] = useState<ServerJSON | null>(null)

  return (
    <>
      <RegistryListContent onSelectServer={setSelectedServer} />
      {/* ServerDetailPanel wired in bead 4 — selectedServer state lives here */}
    </>
  )
}
