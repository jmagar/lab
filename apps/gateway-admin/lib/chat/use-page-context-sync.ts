'use client'

/**
 * usePageContextSync — Syncs the current pathname into ChatSessionProvider.pageContext.
 * Must be rendered inside ChatSessionProvider (consumes ChatSessionActionsContext).
 */

import * as React from 'react'
import { usePathname } from 'next/navigation'
import { useChatSessionActions } from './chat-session-provider'
import type { PageContext } from './chat-session-provider'

export function usePageContextSync(): void {
  const pathname = usePathname()
  const { setPageContext } = useChatSessionActions()

  React.useEffect(() => {
    const ctx: PageContext = { route: pathname }
    setPageContext(ctx)
  }, [pathname, setPageContext])
}
