import { getSessionCsrfToken } from '../auth/session.ts'
import { isStandaloneBearerAuthMode } from '../auth/auth-mode.ts'

export function gatewayHeaders(
  token = process.env.NEXT_PUBLIC_API_TOKEN,
  standaloneBearerAuth = isStandaloneBearerAuthMode(token),
): HeadersInit {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  }
  if (standaloneBearerAuth && token) {
    headers.Authorization = `Bearer ${token}`
  } else {
    const csrfToken = getSessionCsrfToken()
    if (csrfToken) {
      headers['x-csrf-token'] = csrfToken
    }
  }
  return headers
}

export function confirmGatewayParams<T extends object>(params: T): T & { confirm: true } {
  return {
    ...params,
    confirm: true,
  }
}

export function gatewayRequestInit(
  action: string,
  params: object,
  token = process.env.NEXT_PUBLIC_API_TOKEN,
  signal?: AbortSignal,
  standaloneBearerAuth = isStandaloneBearerAuthMode(token),
): RequestInit {
  const credentials: RequestCredentials = standaloneBearerAuth ? 'omit' : 'include'

  return {
    method: 'POST',
    headers: gatewayHeaders(token, standaloneBearerAuth),
    body: JSON.stringify({ action, params }),
    cache: 'no-store',
    credentials,
    signal,
  }
}
