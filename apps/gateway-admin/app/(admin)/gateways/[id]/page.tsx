import { GatewayDetailContent } from '@/components/gateway/gateway-detail-content'

interface GatewayDetailPageProps {
  params: Promise<{ id: string }>
}

export default async function GatewayDetailPage({ params }: GatewayDetailPageProps) {
  const { id } = await params
  return <GatewayDetailContent gatewayId={id} />
}
