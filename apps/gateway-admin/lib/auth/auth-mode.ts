export function hasApiTokenAuth(_token?: string) {
  return false
}

export function isStandaloneBearerAuthMode(
  _token?: string,
) {
  return false
}

export function hasMockDataAuthMode(mockData = process.env.NEXT_PUBLIC_MOCK_DATA) {
  return mockData === 'true'
}

export function shouldBypassBrowserSessionAuth(
  _token?: string,
  mockData = process.env.NEXT_PUBLIC_MOCK_DATA,
) {
  return hasMockDataAuthMode(mockData)
}
