'use client'

/**
 * AdminLayoutClient — client-side wrapper for the admin layout.
 *
 * Hosts ChatSessionProvider, FloatingChatFab, and FloatingChatPopover.
 * Keeps the layout.tsx file a server component (no 'use client' in the layout).
 *
 * Stream lifecycle:
 * - onFirstOpenRef callback is passed to FAB and fired on first click
 * - ChatSessionProvider starts SSE only after first FAB click
 */

import * as React from 'react'
import { ChatSessionProvider } from '@/lib/chat/chat-session-provider'
import { FloatingChatFab } from '@/components/floating-chat-fab'
import { FloatingChatPopover } from '@/components/floating-chat-popover'
import type { PersistConfig } from '@/components/floating-chat-popover'

export function AdminLayoutClient({
  children,
}: {
  children: React.ReactNode
}) {
  const [open, setOpen] = React.useState(() => {
    try {
      const raw = typeof localStorage !== 'undefined'
        ? localStorage.getItem('labby:floating-chat:state')
        : null
      if (!raw) return false
      const parsed = JSON.parse(raw) as { open?: boolean; config?: PersistConfig }
      return Boolean(parsed?.config?.persistOpen && parsed?.open)
    } catch {
      return false
    }
  })

  const [config, setConfig] = React.useState<PersistConfig>({
    persistOpen: true,
    persistPosition: true,
    persistSize: true,
    sendPageContext: false,
  })

  // openModals ref — shared between FAB and CommandPalette
  const openModals = React.useRef<Set<string>>(new Set())

  // First-open ref — ChatSessionProvider registers its callback here
  const onFirstOpenRef = React.useRef<(() => void) | null>(null)
  const hasOpenedOnce = React.useRef(false)

  const [isMobileViewport, setIsMobileViewport] = React.useState(false)

  React.useEffect(() => {
    const media = window.matchMedia('(max-width: 767px)')
    const sync = () => setIsMobileViewport(media.matches)
    sync()
    media.addEventListener('change', sync)
    return () => media.removeEventListener('change', sync)
  }, [])

  const handleToggle = React.useCallback(() => {
    setOpen((prev) => {
      const next = !prev
      // First open: trigger stream start
      if (next && !hasOpenedOnce.current) {
        hasOpenedOnce.current = true
        onFirstOpenRef.current?.()
      }
      // Persist open state
      try {
        const raw = localStorage.getItem('labby:floating-chat:state')
        const parsed = raw ? JSON.parse(raw) : {}
        localStorage.setItem(
          'labby:floating-chat:state',
          JSON.stringify({ ...parsed, open: next }),
        )
      } catch {
        // localStorage unavailable
      }
      return next
    })
  }, [])

  const handleClose = React.useCallback(() => {
    setOpen(false)
    try {
      const raw = localStorage.getItem('labby:floating-chat:state')
      const parsed = raw ? JSON.parse(raw) : {}
      localStorage.setItem(
        'labby:floating-chat:state',
        JSON.stringify({ ...parsed, open: false }),
      )
    } catch {
      // localStorage unavailable
    }
  }, [])

  return (
    <ChatSessionProvider
      onFirstOpenRef={onFirstOpenRef}
      isMobileViewport={isMobileViewport}
    >
      {children}
      <FloatingChatFab
        open={open}
        onToggle={handleToggle}
        openModals={openModals}
      />
      <FloatingChatPopover
        open={open}
        onClose={handleClose}
        config={config}
        onConfigChange={setConfig}
      />
    </ChatSessionProvider>
  )
}
