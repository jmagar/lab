export function hasApiTokenAuth(token = process.env.NEXT_PUBLIC_API_TOKEN) {
  return typeof token === 'string' && token.trim().length > 0
}
