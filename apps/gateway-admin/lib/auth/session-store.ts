export type BrowserSessionState =
  | { status: 'loading' }
  | {
      status: 'authenticated'
      user: {
        sub: string
        email?: string | null
      }
      expiresAt: number
      csrfToken: string
    }
  | { status: 'unauthenticated' }

type SessionPayload =
  | {
      authenticated: true
      user: {
        sub: string
        email?: string | null
      }
      expires_at: number
      csrf_token: string
    }
  | {
      authenticated: false
    }

let currentState: BrowserSessionState = { status: 'loading' }
const listeners = new Set<() => void>()

function emit() {
  for (const listener of listeners) {
    listener()
  }
}

function setState(next: BrowserSessionState) {
  currentState = next
  emit()
}

function normalizePayload(payload: SessionPayload): BrowserSessionState {
  if (!payload.authenticated) {
    return { status: 'unauthenticated' }
  }
  return {
    status: 'authenticated',
    user: payload.user,
    expiresAt: payload.expires_at,
    csrfToken: payload.csrf_token,
  }
}

export function subscribeToBrowserSession(listener: () => void) {
  listeners.add(listener)
  return () => {
    listeners.delete(listener)
  }
}

export function getBrowserSessionState() {
  return currentState
}

export function getSessionCsrfToken() {
  return currentState.status === 'authenticated' ? currentState.csrfToken : undefined
}

export async function loadBrowserSession() {
  let next: BrowserSessionState = { status: 'unauthenticated' }

  try {
    const response = await fetch('/auth/session', {
      cache: 'no-store',
      credentials: 'include',
    })

    if (response.ok) {
      const payload = (await response.json()) as SessionPayload
      next = normalizePayload(payload)
    }
  } catch {
    next = { status: 'unauthenticated' }
  }

  setState(next)
  return next
}

export async function logoutBrowserSession() {
  const csrfToken = getSessionCsrfToken()
  const response = await fetch('/auth/logout', {
    method: 'POST',
    cache: 'no-store',
    credentials: 'include',
    headers: csrfToken
      ? {
          'x-csrf-token': csrfToken,
        }
      : undefined,
  })

  if (!response.ok) {
    throw new Error('Failed to logout browser session')
  }

  setState({ status: 'unauthenticated' })
}

export function __setBrowserSessionStateForTests(state: BrowserSessionState) {
  currentState = state
}
