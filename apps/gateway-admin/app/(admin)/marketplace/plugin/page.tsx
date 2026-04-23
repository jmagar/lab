import { Suspense } from 'react'
import { MarketplacePluginPageClient } from './plugin-page-client'

export default function MarketplacePluginPage() {
  return (
    <Suspense fallback={null}>
      <MarketplacePluginPageClient />
    </Suspense>
  )
}
