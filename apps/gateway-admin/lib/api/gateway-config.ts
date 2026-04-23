export function normalizeGatewayApiBase(baseUrl?: string): string {
  const value = baseUrl || process.env.NEXT_PUBLIC_API_URL || '/v1'
  return value.endsWith('/') ? value.slice(0, -1) : value
}

export function gatewayActionUrl(baseUrl?: string): string {
  return `${normalizeGatewayApiBase(baseUrl)}/gateway`
}

export function extractActionUrl(baseUrl?: string): string {
  return `${normalizeGatewayApiBase(baseUrl)}/extract`
}

export function marketplaceActionUrl(baseUrl?: string): string {
  return `${normalizeGatewayApiBase(baseUrl)}/marketplace`
}

export function gatewayDetailHref(id: string): string {
  return `/gateway/?id=${encodeURIComponent(id)}`
}
