'use client'

// Polls setup.state on mount + on window.focus (debounced 1s trailing
// edge + AbortController dedup) and renders a non-blocking warning
// banner when `draft_stale: true`.

import { useEffect, useState } from 'react'
import { AlertTriangle } from 'lucide-react'

import { setupApi } from '@/lib/api/setup-client'

export function DraftStaleBanner(): React.ReactElement | null {
  const [stale, setStale] = useState(false)

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
        setStale(snapshot.draft_stale)
      } catch {
        // Network errors should not block Settings — keep the banner
        // hidden and try again on the next focus event.
      }
    }

    function schedule(): void {
      if (debounceTimer) clearTimeout(debounceTimer)
      debounceTimer = setTimeout(() => {
        void check()
      }, 1000)
    }

    void check()
    window.addEventListener('focus', schedule)
    return () => {
      cancelled = true
      window.removeEventListener('focus', schedule)
      if (debounceTimer) clearTimeout(debounceTimer)
      inFlight?.abort()
    }
  }, [])

  if (!stale) return null
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
