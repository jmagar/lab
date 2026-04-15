'use client'

import { Suspense } from 'react'
import { useSearchParams } from 'next/navigation'

import { GatewayDetailContent } from '@/components/gateway/gateway-detail-content'

function GatewayDetailPageContent() {
  const searchParams = useSearchParams()
  const gatewayId = searchParams.get('id')

  return <GatewayDetailContent gatewayId={gatewayId} />
}

export default function GatewayDetailPage() {
  return (
    <Suspense fallback={<GatewayDetailContent gatewayId={null} />}>
      <GatewayDetailPageContent />
    </Suspense>
  )
}
