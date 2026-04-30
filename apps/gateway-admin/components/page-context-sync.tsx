'use client'

/**
 * PageContextSync — Drives usePageContextSync inside the ChatSessionProvider tree.
 * Must be a child of AdminLayoutClient (which provides ChatSessionActionsContext).
 */

import { usePageContextSync } from '@/lib/chat/use-page-context-sync'

export function PageContextSync() {
  usePageContextSync()
  return null
}
