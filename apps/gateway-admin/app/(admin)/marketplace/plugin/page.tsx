'use client'

import React from 'react'
import { useSearchParams } from 'next/navigation'
import { PluginDetailContent } from '@/components/marketplace/plugin-detail-content'

export function MarketplacePluginPageContent({ pluginId }: { pluginId: string }) {
  return <PluginDetailContent pluginId={pluginId} />
}

export default function MarketplacePluginPage() {
  const searchParams = useSearchParams()
  const pluginId = searchParams.get('id') ?? ''

  return <MarketplacePluginPageContent pluginId={pluginId} />
}
