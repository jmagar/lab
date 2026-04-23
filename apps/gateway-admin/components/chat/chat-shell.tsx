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
import { ActivityTimeline } from './activity-timeline'
import { deriveTranscriptAndActivity, toProjects } from '@/lib/chat/session-events'
import { useSessionEvents } from '@/lib/chat/use-session-events'
import { normalizeGatewayApiBase } from '@/lib/api/gateway-config'
import { gatewayHeaders } from '@/lib/api/gateway-request'
import { isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'
import type { ACPAgent, ACPRun } from './types'
import type { BridgeSessionSummary, ProviderHealth } from '@/lib/acp/types'

const ACP_AGENT: ACPAgent = {
  id: 'codex',
  name: 'Codex ACP',
  description: 'codex-acp over local ACP bridge',
  version: 'live',
  capabilities: ['tool_use', 'streaming', 'permissions', 'plans'],
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
  const [systemPrompt, setSystemPrompt] = React.useState('')
  const [temperature, setTemperature] = React.useState(0.7)
  const [maxTokens, setMaxTokens] = React.useState(8192)
  const [providerHealth, setProviderHealth] = React.useState<ProviderHealth | null>(null)

  const selectedRun = runs.find((run) => run.id === selectedRunId) ?? null
  const { events, connectionState } = useSessionEvents(selectedRunId)
  const { messages, activity } = React.useMemo(() => deriveTranscriptAndActivity(events), [events])
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
    const payload = (await response.json()) as { sessions: BridgeSessionSummary[] }
    const nextRuns = payload.sessions.map(toRun)
    setRuns(nextRuns)
    setSelectedRunId((current) => current ?? nextRuns[0]?.id ?? null)
  }, [fetchAcp])

  const refreshProvider = React.useCallback(async () => {
    const response = await fetchAcp('/provider')
    const payload = (await response.json()) as { provider: ProviderHealth }
    setProviderHealth(payload.provider)
  }, [fetchAcp])

  React.useEffect(() => {
    void refreshProvider()
    void refreshSessions()
  }, [refreshProvider, refreshSessions])

  React.useEffect(() => {
    if (!providerHealth?.ready) {
      return
    }
    if (runs.length > 0 || selectedRunId) {
      return
    }
    void (async () => {
      const response = await fetchAcp('/sessions', {
        method: 'POST',
        body: JSON.stringify({}),
      })
      const payload = (await response.json()) as { session: BridgeSessionSummary }
      const run = toRun(payload.session)
      setRuns([run])
      setSelectedRunId(run.id)
    })()
  }, [fetchAcp, providerHealth?.ready, runs.length, selectedRunId])

  const handleSelectRun = (runId: string) => {
    setSelectedRunId(runId)
  }

  const handleNewRun = async () => {
    const response = await fetchAcp('/sessions', {
      method: 'POST',
      body: JSON.stringify({}),
    })
    const payload = (await response.json()) as { session: BridgeSessionSummary }
    const run = toRun(payload.session)
    setRuns((current) => [run, ...current])
    setSelectedRunId(run.id)
  }

  const handleSend = async (text: string) => {
    let runId = selectedRunId
    if (!runId) {
      const response = await fetchAcp('/sessions', {
        method: 'POST',
        body: JSON.stringify({}),
      })
      const payload = (await response.json()) as { session: BridgeSessionSummary }
      const run = toRun(payload.session)
      setRuns((current) => [run, ...current])
      setSelectedRunId(run.id)
      runId = run.id
    }

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
    <div className="flex h-screen flex-col overflow-hidden bg-aurora-page-bg">
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-aurora-border-default bg-aurora-nav-bg px-3">
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
            <span className="text-aurora-text-muted/50">{projects[0]?.name}</span>
            <span className="text-aurora-text-muted/30">/</span>
            <span className="max-w-[300px] truncate text-aurora-text-primary">{selectedRun.title}</span>
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

          <div className="flex items-center gap-1 rounded-full border border-aurora-border-default bg-aurora-control-surface px-2 py-0.5">
            <Zap className="size-3 text-aurora-accent-primary/70" />
            <span className="text-[11px] text-aurora-text-muted">{providerHealth?.ready ? 'ACP live' : 'ACP unavailable'}</span>
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
          <SessionSidebar
            projects={projects}
            runs={runs}
            selectedRunId={selectedRunId}
            selectedProjectId="workspace"
            onSelectRun={(runId) => handleSelectRun(runId)}
            onNewRun={() => void handleNewRun()}
          />
        )}

        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          <ActivityTimeline events={activity} connectionState={connectionState} />
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
