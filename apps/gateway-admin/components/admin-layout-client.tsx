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
import { FloatingChatShell } from '@/components/floating-chat-shell'
import { PageContextSync } from '@/components/page-context-sync'
import {
  readPersistedState,
  patchPersistedState,
  DEFAULT_CONFIG,
  type PersistConfig,
} from '@/components/floating-chat-popover'

export function AdminLayoutClient({
  children,
}: {
  children: React.ReactNode
}) {
  const [open, setOpen] = React.useState(() => {
    try {
      const persisted = readPersistedState()
      return Boolean(persisted.config?.persistOpen && persisted.open)
    } catch {
      return false
    }
  })

  // lab-gych.15: initialize config from localStorage
  const [config, setConfig] = React.useState<PersistConfig>(() => {
    try {
      return readPersistedState().config ?? DEFAULT_CONFIG
    } catch {
      return DEFAULT_CONFIG
    }
  })

  // openModals ref — shared between FAB and CommandPalette
  const openModals = React.useRef<Set<string>>(new Set())

  // First-open ref — ChatSessionProvider registers its callback here
  const onFirstOpenRef = React.useRef<(() => void) | null>(null)
  const hasOpenedOnce = React.useRef(false)

  // lab-gych.7: initialize shellMounted from the same localStorage check as `open`
  const [shellMounted, setShellMounted] = React.useState(() => {
    try {
      const persisted = readPersistedState()
      return Boolean(persisted.config?.persistOpen && persisted.open)
    } catch {
      return false
    }
  })

  const [isMobileViewport, setIsMobileViewport] = React.useState(false)

  React.useEffect(() => {
    const media = window.matchMedia('(max-width: 767px)')
    const sync = () => setIsMobileViewport(media.matches)
    sync()
    media.addEventListener('change', sync)
    return () => media.removeEventListener('change', sync)
  }, [])

  // lab-gych.7: when hydrating with open=true on first mount, seed hasOpenedOnce
  // and fire the stream-start callback
  React.useEffect(() => {
    if (open && !hasOpenedOnce.current) {
      hasOpenedOnce.current = true
      onFirstOpenRef.current?.()
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []) // intentionally runs once on mount

  // lab-gych.8: side effects (stream start, shellMounted, localStorage write) live
  // in a useEffect watching `open`, NOT inside the setState updater
  React.useEffect(() => {
    // First open: trigger stream start and mount shell
    if (open && !hasOpenedOnce.current) {
      hasOpenedOnce.current = true
      onFirstOpenRef.current?.()
      setShellMounted(true)
    } else if (open && !shellMounted) {
      setShellMounted(true)
    }

    // Persist open state
    patchPersistedState({ open })
  }, [open]) // eslint-disable-line react-hooks/exhaustive-deps

  const handleToggle = React.useCallback(() => {
    // lab-gych.8: pure updater — no side effects inside setState
    setOpen((prev) => !prev)
  }, [])

  const handleClose = React.useCallback(() => {
    setOpen(false)
  }, [])

  return (
    <ChatSessionProvider
      onFirstOpenRef={onFirstOpenRef}
      isMobileViewport={isMobileViewport}
    >
      <PageContextSync />
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
      >
        {/* FloatingChatShell mounts lazily on first FAB click, stays mounted permanently */}
        {shellMounted && <FloatingChatShell config={config} />}
      </FloatingChatPopover>
    </ChatSessionProvider>
  )
}
