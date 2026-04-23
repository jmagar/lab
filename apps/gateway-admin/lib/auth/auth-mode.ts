export function hasApiTokenAuth(token = process.env.NEXT_PUBLIC_API_TOKEN) {
  void token
  return false
}

export function isStandaloneBearerAuthMode(
  token = process.env.NEXT_PUBLIC_API_TOKEN,
) {
  void token
  return false
}

export function hasMockDataAuthMode(mockData = process.env.NEXT_PUBLIC_MOCK_DATA) {
  return mockData === 'true'
}

export function shouldBypassBrowserSessionAuth(
  token = process.env.NEXT_PUBLIC_API_TOKEN,
  mockData = process.env.NEXT_PUBLIC_MOCK_DATA,
) {
  void token
  return hasMockDataAuthMode(mockData)
}
