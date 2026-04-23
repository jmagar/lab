'use client'

import { useState } from 'react'
import { useSWRConfig } from 'swr'
import { RegistryListContent } from '@/components/registry/registry-list-content'
import { ServerDetailPanel } from '@/components/registry/server-detail-panel'
import { LAB_REGISTRY_META_KEY, REGISTRY_META_KEY } from '@/lib/types/registry'
import { REGISTRY_SERVER_KEY, REGISTRY_SERVERS_KEY } from '@/lib/hooks/use-registry'
import type { ServerResponse } from '@/lib/types/registry'

export default function RegistryPage() {
  const [selectedResponse, setSelectedResponse] = useState<ServerResponse | null>(null)
  const { mutate } = useSWRConfig()

  const extensions = selectedResponse?._meta?.[REGISTRY_META_KEY] ?? null
  const labMetadata = selectedResponse?._meta?.[LAB_REGISTRY_META_KEY] ?? null

  const applyLabMetadata = (metadata: ServerResponse['_meta'][typeof LAB_REGISTRY_META_KEY] | null) => {
    setSelectedResponse((current) => current
      ? {
          ...current,
          _meta: {
            ...(current._meta ?? {}),
            [LAB_REGISTRY_META_KEY]: metadata,
          },
        }
      : current)

    void mutate(
      (key) => Array.isArray(key) && key[0] === REGISTRY_SERVERS_KEY,
      (current: { servers?: ServerResponse[] }[] | undefined) => {
        if (!current) return current
        return current.map((page) => ({
          ...page,
          servers: page.servers?.map((serverResponse) =>
            serverResponse.server.name === selectedResponse?.server.name
              ? {
                  ...serverResponse,
                  _meta: {
                    ...(serverResponse._meta ?? {}),
                    [LAB_REGISTRY_META_KEY]: metadata,
                  },
                }
              : serverResponse,
          ) ?? [],
        }))
      },
      false,
    )
    void mutate((key) => Array.isArray(key) && key[0] === REGISTRY_SERVERS_KEY)

    if (selectedResponse) {
      void mutate(
        [REGISTRY_SERVER_KEY, selectedResponse.server.name],
        (current: ServerResponse | undefined) => current
          ? {
              ...current,
              _meta: {
                ...(current._meta ?? {}),
                [LAB_REGISTRY_META_KEY]: metadata,
              },
            }
          : current,
        false,
      )
    }
  }

  return (
    <>
      <RegistryListContent onSelectServer={setSelectedResponse} />
      <ServerDetailPanel
        server={selectedResponse?.server ?? null}
        extensions={extensions}
        labMetadata={labMetadata}
        onLabMetadataChange={applyLabMetadata}
        onClose={() => setSelectedResponse(null)}
      />
    </>
  )
}
