import { getSessionCsrfToken } from '../auth/session.ts'

export function gatewayHeaders(
  _token?: string,
  _standaloneBearerAuth = false,
): HeadersInit {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  }
  const csrfToken = getSessionCsrfToken()
  if (csrfToken) {
    headers['x-csrf-token'] = csrfToken
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
  _token?: string,
  signal?: AbortSignal,
  _standaloneBearerAuth = false,
): RequestInit {
  return {
    method: 'POST',
    headers: gatewayHeaders(),
    body: JSON.stringify({ action, params }),
    cache: 'no-store',
    credentials: 'include',
    signal,
  }
}
