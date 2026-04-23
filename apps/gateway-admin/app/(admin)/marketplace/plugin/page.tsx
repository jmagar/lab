'use client'

import { useSearchParams } from 'next/navigation'
import { PluginDetailContent } from '@/components/marketplace/plugin-detail-content'

export const metadata = { title: 'Marketplace Plugin — Labby' }

export default function MarketplacePluginPage() {
  const searchParams = useSearchParams()
  const pluginId = searchParams.get('id') ?? ''

  return <PluginDetailContent pluginId={pluginId} />
}
