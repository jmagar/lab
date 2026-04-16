export function hasApiTokenAuth(token = process.env.NEXT_PUBLIC_API_TOKEN) {
  return typeof token === 'string' && token.trim().length > 0
}

export function hasMockDataAuthMode(mockData = process.env.NEXT_PUBLIC_MOCK_DATA) {
  return mockData === 'true'
}

export function shouldBypassBrowserSessionAuth(
  token = process.env.NEXT_PUBLIC_API_TOKEN,
  mockData = process.env.NEXT_PUBLIC_MOCK_DATA,
) {
  return hasApiTokenAuth(token) || hasMockDataAuthMode(mockData)
}
