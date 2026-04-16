'use client'

import { useSyncExternalStore } from 'react'

export {
  __setBrowserSessionStateForTests,
  getBrowserSessionState,
  getSessionCsrfToken,
  loadBrowserSession,
  logoutBrowserSession,
  subscribeToBrowserSession,
  type BrowserSessionState,
} from './session-store'
import { getBrowserSessionState, subscribeToBrowserSession } from './session-store.ts'

export function useBrowserSession() {
  return useSyncExternalStore(
    subscribeToBrowserSession,
    getBrowserSessionState,
    getBrowserSessionState,
  )
}
