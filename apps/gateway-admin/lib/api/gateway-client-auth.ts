import { hasApiTokenAuth } from '../auth/auth-mode.ts'
import { getSessionCsrfToken, loadBrowserSession } from '../auth/session-store.ts'
import { GatewayApiError, gatewayActionCore } from './gateway-client-core.ts'

const SESSION_REFRESH_STATUSES = new Set([401, 403, 422])

// Trigger a silent browser-session refresh only when the backend signals an
// authentication/CSRF problem. `validation_failed` is reused on the backend
// for CSRF rejection (see `crates/lab/src/api/router.rs::csrf_error_response`)
// so we narrow with: auth-status code AND the literal word "csrf" in the
// message AND the client had actually attempted session auth. Without all
// three, a plain 422 from a mistyped param would trigger a session reload.
function shouldRefreshBrowserSession(
  error: GatewayApiError,
  attemptedSessionAuth: boolean,
) {
  if (hasApiTokenAuth()) {
    return false
  }

  if (error.code === 'auth_failed') {
    return true
  }

  if (!attemptedSessionAuth || error.code !== 'validation_failed') {
    return false
  }

  const isAuthStatus = SESSION_REFRESH_STATUSES.has(error.status)
  const isCsrfMessage = error.message.toLowerCase().includes('csrf')
  return isAuthStatus && isCsrfMessage
}

export async function gatewayAction<T>(
  action: string,
  params: object,
  signal?: AbortSignal,
): Promise<T> {
  const attemptedSessionAuth = Boolean(getSessionCsrfToken())

  try {
    return await gatewayActionCore<T>(action, params, signal)
  } catch (error) {
    if (
      error instanceof GatewayApiError &&
      shouldRefreshBrowserSession(error, attemptedSessionAuth)
    ) {
      await loadBrowserSession().catch(() => undefined)
    }
    throw error
  }
}
