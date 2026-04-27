'use client'

/**
 * FloatingChatShell — Wires all /chat features into the floating chat popover.
 *
 * Lazy-mounts on first FAB click, stays mounted permanently.
 * Consumes the 4 ChatSession contexts provided by ChatSessionProvider.
 *
 * Lifecycle:
 * - Provider starts SSE on first FAB click (streamEnabled = true)
 * - This shell mounts when the provider signals first-open
 * - Visibility is controlled by the popover (CSS visibility:hidden when closed)
 * - Dual-stream impossible: FAB hidden on /chat, FloatingChatShell never mounts there
 *
 * pageContext:
 * - Reads sendPageContext from config prop
 * - When enabled AND pageContext is non-null: includes pageContext in prompt body
 * - When disabled or null: omits field entirely (zero token cost)
 */

import * as React from 'react'
import { Plus, SidebarOpen, Zap } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { SessionSidebar } from '@/components/chat/session-sidebar'
import { MessageThread } from '@/components/chat/message-thread'
import { ChatInput } from '@/components/chat/chat-input'
import { ACP_AGENT, ensurePromptRunId } from '@/lib/chat/use-chat-session-controller'
import { createAcpFetcher } from '@/lib/acp/fetch'
import {
  useChatSessionData,
  useChatSessionActions,
  useChatSessionConnection,
  useChatSessionStream,
} from '@/lib/chat/chat-session-provider'
import type { PersistConfig } from '@/components/floating-chat-popover'
import type { ChatInputPayload } from '@/components/chat/chat-input'
import type { AttachmentRef } from '@/lib/fs/types'

export type FloatingChatShellProps = {
  config?: PersistConfig
}

export function FloatingChatShell({
  config,
}: FloatingChatShellProps) {
  // ---- Context consumers ----
  const { runs, selectedRun, selectedRunId, providerHealth, providers, agents, projects, pageContext } =
    useChatSessionData()
  const { createSession, selectRun, refreshSessions, selectAgent } = useChatSessionActions()
  const { connectionState } = useChatSessionConnection()
  const { messages } = useChatSessionStream()

  // ---- Local state ----
  const [sessionPanelOpen, setSessionPanelOpen] = React.useState(true)
  const [isMobileViewport, setIsMobileViewport] = React.useState(false)

  const providerReady = Boolean(providerHealth?.ready)

  const selectedAgent = selectedRun
    ? (agents.find((agent) => agent.id === selectedRun.provider) ?? {
        ...ACP_AGENT,
        id: selectedRun.provider,
        name: selectedRun.provider,
      })
    : (agents[0] ?? ACP_AGENT)

  // ---- Mobile viewport detection ----
  React.useEffect(() => {
    const media = window.matchMedia('(max-width: 767px)')
    const sync = () => {
      setIsMobileViewport(media.matches)
      setSessionPanelOpen((open) => (media.matches ? false : open))
    }
    sync()
    media.addEventListener('change', sync)
    return () => media.removeEventListener('change', sync)
  }, [])

  // ---- sendPrompt (reads pageContext from provider + config) ----
  const sendPrompt = React.useCallback(
    async (payload: ChatInputPayload) => {
      try {
        const runId = await ensurePromptRunId(selectedRunId, createSession, isMobileViewport)
        const fetchAcp = createAcpFetcher()

        // Include pageContext only if config.sendPageContext is true AND context is non-null
        const includePageContext = config?.sendPageContext && pageContext !== null

        const body: {
          prompt: string
          attachments?: AttachmentRef[]
          pageContext?: NonNullable<typeof pageContext>
        } = {
          prompt: payload.text,
          ...(payload.attachments.length > 0 && { attachments: payload.attachments }),
          ...(includePageContext && pageContext && { pageContext }),
        }

        const response = await fetchAcp(`/sessions/${runId}/prompt`, {
          method: 'POST',
          body: JSON.stringify(body),
        })

        if (!response.ok) {
          // Non-fatal — providerHealth messaging handles this
          return
        }

        await refreshSessions()
      } catch {
        // Keep UI responsive; provider health message carries the failure detail
      }
    },
    [config?.sendPageContext, createSession, isMobileViewport, pageContext, refreshSessions, selectedRunId],
  )

  const createRun = React.useCallback(async () => {
    try {
      await createSession()
    } catch {
      // providerHealth.message carries the failure detail
    }
  }, [createSession])

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      {/* ---- Compact header (48px) ---- */}
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-aurora-border-default bg-aurora-nav-bg px-2.5">
        <TooltipProvider delayDuration={400}>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                aria-label="Toggle sessions"
                onClick={() => setSessionPanelOpen((open) => !open)}
                className={cn(
                  'size-7 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
                  sessionPanelOpen && 'bg-aurora-hover-bg text-aurora-text-primary',
                )}
              >
                <SidebarOpen className="size-3.5" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom" className="text-xs">Toggle sessions</TooltipContent>
          </Tooltip>
        </TooltipProvider>

        {selectedRun && (
          <div className="ml-1 flex min-w-0 flex-1 items-center gap-1.5 text-[12px] text-aurora-text-muted">
            <span className="max-w-[160px] truncate text-aurora-text-primary">{selectedRun.title}</span>
          </div>
        )}

        <div className="ml-auto flex items-center gap-1.5">
          <TooltipProvider delayDuration={400}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Start new session"
                  onClick={() => void createRun()}
                  disabled={!providerReady}
                  className="size-7 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
                >
                  <Plus className="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom" className="text-xs">New session</TooltipContent>
            </Tooltip>
          </TooltipProvider>

          <div
            className="flex items-center gap-1 rounded-full border border-aurora-border-default bg-aurora-control-surface px-1.5 py-0.5"
            title={providerReady ? 'ACP live' : 'ACP unavailable'}
          >
            <Zap className="size-3 text-aurora-accent-primary/70" />
            <span className="text-[11px] text-aurora-text-muted">
              {providerReady ? 'ACP' : '—'}
            </span>
          </div>

        </div>
      </header>

      {/* ---- Session sidebar + message thread ---- */}
      <div className="flex min-h-0 flex-1">
        {sessionPanelOpen && (
          <SessionSidebar
            className="hidden w-[180px] shrink-0 md:flex"
            projects={projects}
            runs={runs}
            selectedRunId={selectedRunId}
            selectedProjectId="workspace"
            onSelectRun={(runId, _projectId) => selectRun(runId)}
            onNewRun={() => void createRun()}
          />
        )}

        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          {/* Message thread — React.memo'd, only re-renders when messages changes */}
          <MessageThread run={selectedRun} messages={messages} />

          {/* Chat input */}
          <ChatInput
            onSend={sendPrompt}
            disabled={!providerReady}
            selectedAgent={selectedAgent}
            agents={agents.length > 0 ? agents : [selectedAgent]}
            onSelectAgent={selectAgent}
          />
        </div>

      </div>
    </div>
  )
}
