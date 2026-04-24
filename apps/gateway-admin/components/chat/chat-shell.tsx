'use client'

import * as React from 'react'
import { Plus, Settings2, SidebarOpen, Zap } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { SidebarTrigger } from '@/components/ui/sidebar'
import { Separator } from '@/components/ui/separator'
import { SessionSidebar } from './session-sidebar'
import { MessageThread } from './message-thread'
import { ChatInput } from './chat-input'
import { SettingsPanel } from './settings-panel'
import { deriveTranscriptAndActivity, toProjects } from '@/lib/chat/session-events'
import { useSessionEvents } from '@/lib/chat/use-session-events'
import { normalizeGatewayApiBase } from '@/lib/api/gateway-config'
import { gatewayHeaders } from '@/lib/api/gateway-request'
import { isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'
import type { ACPAgent, ACPRun } from './types'
import type { BridgeSessionSummary, ProviderHealth } from '@/lib/acp/types'

const USE_MOCK_DATA = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

const ACP_AGENT: ACPAgent = {
  id: 'codex',
  name: 'Codex ACP',
  description: 'codex-acp over local ACP bridge',
  version: 'live',
  capabilities: ['tool_use', 'streaming', 'permissions', 'plans'],
}

type SessionCreationIntent = 'bootstrap' | 'manual' | 'send'
type CreateSessionOptions = { closeSessionPanel?: boolean }
type CreateSessionFn = (options?: CreateSessionOptions) => Promise<ACPRun>

export function shouldAutoCreateInitialRun(
  providerReady: boolean,
  runCount: number,
  selectedRunId: string | null,
) {
  return providerReady && runCount === 0 && !selectedRunId
}

export function integrateCreatedRun(current: ACPRun[], run: ACPRun) {
  return [run, ...current.filter((existing) => existing.id !== run.id)]
}

export function sessionCreationOptionsForIntent(
  intent: SessionCreationIntent,
  isMobileViewport: boolean,
): CreateSessionOptions | undefined {
  if (intent === 'bootstrap') {
    return undefined
  }

  return { closeSessionPanel: isMobileViewport }
}

export async function createSessionForIntent(
  createSession: CreateSessionFn,
  intent: SessionCreationIntent,
  isMobileViewport: boolean,
) {
  return createSession(sessionCreationOptionsForIntent(intent, isMobileViewport))
}

export async function ensurePromptRunId(
  selectedRunId: string | null,
  createSession: CreateSessionFn,
  isMobileViewport: boolean,
) {
  if (selectedRunId) {
    return selectedRunId
  }

  const run = await createSessionForIntent(createSession, 'send', isMobileViewport)
  return run.id
}

function toRun(session: BridgeSessionSummary): ACPRun {
  return {
    id: session.id,
    projectId: 'workspace',
    agentId: session.provider,
    provider: session.provider,
    title: session.title,
    createdAt: new Date(session.createdAt),
    updatedAt: new Date(session.updatedAt),
    status: session.status,
    providerSessionId: session.providerSessionId,
    cwd: session.cwd,
  }
}

export function ChatShell() {
  const acpBase = React.useMemo(() => `${normalizeGatewayApiBase()}/acp`, [])
  const standaloneBearerAuth = React.useMemo(() => isStandaloneBearerAuthMode(), [])
  const requestCredentials = React.useMemo<RequestCredentials>(
    () => (standaloneBearerAuth ? 'omit' : 'include'),
    [standaloneBearerAuth],
  )
  const [runs, setRuns] = React.useState<ACPRun[]>([])
  const [selectedRunId, setSelectedRunId] = React.useState<string | null>(null)
  const [sessionPanelOpen, setSessionPanelOpen] = React.useState(true)
  const [settingsOpen, setSettingsOpen] = React.useState(false)
  const [isMobileViewport, setIsMobileViewport] = React.useState(false)
  const [systemPrompt, setSystemPrompt] = React.useState('')
  const [temperature, setTemperature] = React.useState(0.7)
  const [maxTokens, setMaxTokens] = React.useState(8192)
  const [providerHealth, setProviderHealth] = React.useState<ProviderHealth | null>(null)

  const selectedRun = runs.find((run) => run.id === selectedRunId) ?? null
  const { events, connectionState } = useSessionEvents(selectedRunId)
  const { messages } = React.useMemo(() => deriveTranscriptAndActivity(events), [events])
  const projects = React.useMemo(() => toProjects(runs.map((run) => ({
    id: run.id,
    providerSessionId: run.providerSessionId,
    provider: 'codex',
    title: run.title,
    cwd: run.cwd,
    createdAt: run.createdAt.toISOString(),
    updatedAt: run.updatedAt.toISOString(),
    status: run.status,
    agentName: ACP_AGENT.name,
    agentVersion: ACP_AGENT.version,
  }))), [runs])

  const fetchAcp = React.useCallback(async (path: string, init?: RequestInit) => {
    const headers = new Headers(init?.headers ?? {})
    const authHeaders = gatewayHeaders()
    for (const [key, value] of Object.entries(authHeaders)) {
      headers.set(key, value)
    }

    return fetch(`${acpBase}${path}`, {
      ...init,
      headers,
      credentials: requestCredentials,
      cache: 'no-store',
    })
  }, [acpBase, requestCredentials])

  const refreshSessions = React.useCallback(async () => {
    const response = await fetchAcp('/sessions')
    if (!response.ok) {
      setRuns([])
      setSelectedRunId(null)
      return
    }
    const payload = (await response.json()) as { sessions: BridgeSessionSummary[] }
    const nextRuns = payload.sessions.map(toRun)
    setRuns(nextRuns)
    setSelectedRunId((current) => current ?? nextRuns[0]?.id ?? null)
  }, [fetchAcp])

  const refreshProvider = React.useCallback(async () => {
    const response = await fetchAcp('/provider')
    if (!response.ok) {
      setProviderHealth({
        provider: 'codex',
        ready: false,
        command: '',
        args: [],
        message: 'ACP provider unavailable.',
      })
      return
    }
    const payload = (await response.json()) as { provider: ProviderHealth }
    setProviderHealth(payload.provider)
  }, [fetchAcp])

  const createSession = React.useCallback(
    async (options?: { closeSessionPanel?: boolean }) => {
      const response = await fetchAcp('/sessions', {
        method: 'POST',
        body: JSON.stringify({}),
      })
      const payload = (await response.json()) as { session: BridgeSessionSummary }
      const run = toRun(payload.session)
      setRuns((current) => integrateCreatedRun(current, run))
      setSelectedRunId(run.id)
      if (options?.closeSessionPanel) {
        setSessionPanelOpen(false)
      }
      return run
    },
    [fetchAcp],
  )

  React.useEffect(() => {
    if (USE_MOCK_DATA) {
      setProviderHealth({
        provider: 'codex',
        ready: false,
        command: '',
        args: [],
        message: 'ACP unavailable in mock preview.',
      })
      setRuns([])
      setSelectedRunId(null)
      return
    }

    void refreshProvider()
    void refreshSessions()
  }, [refreshProvider, refreshSessions])

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

  React.useEffect(() => {
    if (!shouldAutoCreateInitialRun(Boolean(providerHealth?.ready), runs.length, selectedRunId)) {
      return
    }
    void (async () => {
      await createSessionForIntent(createSession, 'bootstrap', false)
    })()
  }, [createSession, providerHealth?.ready, runs.length, selectedRunId])

  const handleSelectRun = (runId: string) => {
    setSelectedRunId(runId)
    if (isMobileViewport) {
      setSessionPanelOpen(false)
    }
  }

  const handleNewRun = async () => {
    await createSessionForIntent(createSession, 'manual', isMobileViewport)
  }

  const handleSend = async (text: string) => {
    const runId = await ensurePromptRunId(selectedRunId, createSession, isMobileViewport)

    await fetchAcp(`/sessions/${runId}/prompt`, {
      method: 'POST',
      body: JSON.stringify({ prompt: text }),
    })
    await refreshSessions()
  }

  const selectedAgent = selectedRun
    ? {
        ...ACP_AGENT,
        id: selectedRun.provider,
        name: `${selectedRun.provider} ACP`,
      }
    : {
        ...ACP_AGENT,
        id: providerHealth?.provider ?? ACP_AGENT.id,
        name: `${providerHealth?.provider ?? ACP_AGENT.id} ACP`,
      }

  return (
    <div className="flex h-dvh min-h-0 flex-col overflow-hidden bg-aurora-page-bg">
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-aurora-border-default bg-aurora-nav-bg px-2.5 sm:px-3">
        <SidebarTrigger
          aria-label="Toggle app sidebar"
          className="-ml-1 text-aurora-text-muted/60 hover:text-aurora-text-primary"
        />
        <Separator orientation="vertical" className="h-4 bg-aurora-border-default" />

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
          <div className="ml-1 flex items-center gap-1.5 text-[12px] text-aurora-text-muted">
            <span className="hidden text-aurora-text-muted/50 sm:block">{projects[0]?.name}</span>
            <span className="hidden text-aurora-text-muted/30 sm:block">/</span>
            <span className="max-w-[180px] truncate text-aurora-text-primary sm:max-w-[300px]">{selectedRun.title}</span>
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
                  onClick={() => void handleNewRun()}
                  className="size-7 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary"
                >
                  <Plus className="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom" className="text-xs">New session</TooltipContent>
            </Tooltip>
          </TooltipProvider>

          <div
            className="flex items-center gap-1 rounded-full border border-aurora-border-default bg-aurora-control-surface px-1.5 py-0.5 sm:px-2"
            title={providerHealth?.ready ? 'ACP live' : 'ACP unavailable'}
          >
            <Zap className="size-3 text-aurora-accent-primary/70" />
            <span className="hidden text-[11px] text-aurora-text-muted sm:inline">
              {providerHealth?.ready ? 'ACP live' : 'ACP unavailable'}
            </span>
          </div>

          <TooltipProvider delayDuration={400}>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={settingsOpen ? 'Close settings' : 'Open settings'}
                  onClick={() => setSettingsOpen((open) => !open)}
                  className={cn(
                    'size-7 rounded text-aurora-text-muted/60 hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
                    settingsOpen && 'bg-aurora-hover-bg text-aurora-text-primary',
                  )}
                >
                  <Settings2 className="size-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="bottom" className="text-xs">Settings</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        {sessionPanelOpen && (
          <>
            {isMobileViewport && (
              <button
                type="button"
                aria-label="Close session drawer"
                className="fixed inset-0 z-30 bg-aurora-page-bg/70 backdrop-blur-[2px] md:hidden"
                onClick={() => setSessionPanelOpen(false)}
              />
            )}
            <div
              className={cn(
                'min-h-0 shrink-0',
                isMobileViewport
                  ? 'absolute inset-y-0 left-0 z-40 w-[min(88vw,320px)] md:hidden'
                  : 'relative z-0 hidden md:block',
              )}
            >
              <SessionSidebar
                className="shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong)] md:shadow-none"
                projects={projects}
                runs={runs}
                selectedRunId={selectedRunId}
                selectedProjectId="workspace"
                onSelectRun={(runId) => handleSelectRun(runId)}
                onNewRun={() => void handleNewRun()}
              />
            </div>
          </>
        )}

        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          <MessageThread run={selectedRun} messages={messages} />
          <ChatInput
            onSend={handleSend}
            disabled={!providerHealth?.ready}
            selectedAgent={selectedAgent}
            agents={[selectedAgent]}
            onSelectAgent={() => {}}
          />
        </div>

        {settingsOpen && (
          <SettingsPanel
            agent={selectedAgent}
            onClose={() => setSettingsOpen(false)}
            systemPrompt={systemPrompt}
            onSystemPromptChange={setSystemPrompt}
            temperature={temperature}
            onTemperatureChange={setTemperature}
            maxTokens={maxTokens}
            onMaxTokensChange={setMaxTokens}
          />
        )}
      </div>
    </div>
  )
}
