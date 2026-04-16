export function hasApiTokenAuth(token = process.env.NEXT_PUBLIC_API_TOKEN) {
  return typeof token === 'string' && token.trim().length > 0
}

export function hasStandaloneBearerAuthOverride(
  mode = process.env.NEXT_PUBLIC_STANDALONE_BEARER_AUTH,
) {
  return mode === 'true'
}

export function isStandaloneBearerAuthMode(
  token = process.env.NEXT_PUBLIC_API_TOKEN,
  mode = process.env.NEXT_PUBLIC_STANDALONE_BEARER_AUTH,
) {
  return hasApiTokenAuth(token) && hasStandaloneBearerAuthOverride(mode)
}

export function hasMockDataAuthMode(mockData = process.env.NEXT_PUBLIC_MOCK_DATA) {
  return mockData === 'true'
}

export function shouldBypassBrowserSessionAuth(
  token = process.env.NEXT_PUBLIC_API_TOKEN,
  mockData = process.env.NEXT_PUBLIC_MOCK_DATA,
  standaloneBearerAuth = process.env.NEXT_PUBLIC_STANDALONE_BEARER_AUTH,
) {
  return isStandaloneBearerAuthMode(token, standaloneBearerAuth) || hasMockDataAuthMode(mockData)
}
