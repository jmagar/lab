'use client'

// Polls setup.state on mount + on window.focus (debounced 1s trailing
// edge + AbortController dedup) and renders a non-blocking warning
// banner when `draft_stale: true`.

import { useEffect, useState } from 'react'
import { AlertTriangle } from 'lucide-react'

import { setupApi } from '@/lib/api/setup-client'

type Status = 'unknown' | 'fresh' | 'stale' | 'unavailable'

export function DraftStaleBanner(): React.ReactElement | null {
  const [status, setStatus] = useState<Status>('unknown')

  useEffect(() => {
    let cancelled = false
    let inFlight: AbortController | null = null
    let debounceTimer: ReturnType<typeof setTimeout> | null = null

    async function check(): Promise<void> {
      inFlight?.abort()
      const controller = new AbortController()
      inFlight = controller
      try {
        const snapshot = await setupApi.state(controller.signal)
        if (cancelled || controller.signal.aborted) return
        setStatus(snapshot.draft_stale ? 'stale' : 'fresh')
      } catch (err) {
        if (cancelled || controller.signal.aborted) return
        // AbortError is expected churn (a newer check superseded this one)
        // and is silent. Anything else means the gateway is unreachable
        // or returning errors — surface that as 'unavailable' so users
        // know draft-stale detection is offline rather than silently
        // assuming everything is fine.
        if (err instanceof Error && err.name === 'AbortError') return
        console.warn('DraftStaleBanner: setup.state failed', err)
        setStatus('unavailable')
      }
    }

    function schedule(): void {
      if (debounceTimer) clearTimeout(debounceTimer)
      debounceTimer = setTimeout(() => {
        void check()
      }, 1000)
    }

    function onVisibility(): void {
      if (document.visibilityState === 'visible') schedule()
    }

    void check()
    window.addEventListener('focus', schedule)
    // visibilitychange covers tab-switch on browsers where 'focus' doesn't
    // fire when switching between tabs in the same window (Chrome on
    // mobile, some multi-tab desktop workflows).
    document.addEventListener('visibilitychange', onVisibility)
    return () => {
      cancelled = true
      window.removeEventListener('focus', schedule)
      document.removeEventListener('visibilitychange', onVisibility)
      if (debounceTimer) clearTimeout(debounceTimer)
      inFlight?.abort()
    }
  }, [])

  if (status === 'unknown' || status === 'fresh') return null
  if (status === 'unavailable') {
    return (
      <div className="flex items-start gap-2 rounded-md border border-muted bg-muted/30 p-3 text-sm text-muted-foreground">
        <AlertTriangle className="h-4 w-4 mt-0.5 flex-shrink-0" />
        <div>
          <p className="font-medium text-foreground">Draft state check unavailable.</p>
          <p>
            Could not reach the lab gateway. Concurrent-edit detection is
            offline — saving here may overwrite changes from another session
            without warning.
          </p>
        </div>
      </div>
    )
  }
  return (
    <div className="flex items-start gap-2 rounded-md border border-amber-500/50 bg-amber-50 p-3 text-sm text-amber-900 dark:bg-amber-950/40 dark:text-amber-200">
      <AlertTriangle className="h-4 w-4 mt-0.5 flex-shrink-0" />
      <div>
        <p className="font-medium">Draft is stale.</p>
        <p>
          Another session has unsaved changes to <code>~/.lab/.env</code>.
          Review or discard before continuing — saving here may conflict.
        </p>
      </div>
    </div>
  )
}
