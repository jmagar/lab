'use client'

import { useSearchParams } from 'next/navigation'
import { PluginDetailContent } from '@/components/marketplace/plugin-detail-content'

export function MarketplacePluginPageClient() {
  const searchParams = useSearchParams()
  const pluginId = searchParams.get('id') ?? ''

  return <PluginDetailContent pluginId={pluginId} />
}
