import { MarketplaceListContent } from '@/components/marketplace/marketplace-list-content'

export const metadata = { title: 'Marketplace Preview — Labby' }

export default function DevMarketplacePage() {
  return <MarketplaceListContent readOnlyPreview />
}
