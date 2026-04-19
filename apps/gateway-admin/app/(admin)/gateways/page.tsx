import { GatewayListContent } from '@/components/gateway/gateway-list-content'
import { UpstreamOauthSection } from '@/components/upstream-oauth/upstream-oauth-section'

export default function GatewaysPage() {
  return (
    <>
      <GatewayListContent />
      <div className="px-6 pb-6">
        <UpstreamOauthSection />
      </div>
    </>
  )
}
