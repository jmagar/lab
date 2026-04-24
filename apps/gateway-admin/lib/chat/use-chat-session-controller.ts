'use client'

import * as React from 'react'
import { normalizeGatewayApiBase } from '@/lib/api/gateway-config'
import { gatewayHeaders } from '@/lib/api/gateway-request'
import { isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'
import { toProjects } from './session-events'
import { useSessionEvents } from './use-session-events'
import type { ACPAgent, ACPRun } from '@/components/chat/types'
import type { BridgeSessionSummary, ProviderHealth } from '@/lib/acp/types'

const USE_MOCK_DATA = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

export const ACP_AGENT: ACPAgent = {
  id: 'codex',
  name: 'Codex ACP',
  description: 'codex-acp over local ACP bridge',
  version: 'live',
  capabilities: ['tool_use', 'streaming', 'permissions', 'plans'],
}

type ProviderListPayload = {
  providers?: Array<{
    name?: string
    available?: boolean
    version?: string | null
    error?: string | null
  }>
  provider?: ProviderHealth
}

type SessionCreatePayload = {
  session?: BridgeSessionSummary
} & BridgeSessionSummary

export type SessionCreationIntent = 'bootstrap' | 'manual' | 'send'
export type CreateSessionOptions = { closeSessionPanel?: boolean }
export type CreateSessionFn = (options?: CreateSessionOptions) => Promise<ACPRun>

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

function normalizeProviderHealth(payload: ProviderListPayload): ProviderHealth {
  if (payload.provider) {
    return payload.provider
  }

  const provider = payload.providers?.[0]
  return {
    provider: provider?.name ?? 'codex',
    ready: Boolean(provider?.available),
    command: '',
    args: [],
    message: provider?.error ?? '',
  }
}

function extractCreatedSession(payload: SessionCreatePayload): BridgeSessionSummary {
  return payload.session ?? payload
}

export function useChatSessionController(options: {
  isMobileViewport: boolean
  onSessionPanelClose?: () => void
}) {
  const { isMobileViewport, onSessionPanelClose } = options
  const acpBase = React.useMemo(() => `${normalizeGatewayApiBase()}/acp`, [])
  const standaloneBearerAuth = React.useMemo(() => isStandaloneBearerAuthMode(), [])
  const requestCredentials = React.useMemo<RequestCredentials>(
    () => (standaloneBearerAuth ? 'omit' : 'include'),
    [standaloneBearerAuth],
  )
  const [runs, setRuns] = React.useState<ACPRun[]>([])
  const [selectedRunId, setSelectedRunId] = React.useState<string | null>(null)
  const [providerHealth, setProviderHealth] = React.useState<ProviderHealth | null>(null)
  const { events, messages, activity, connectionState } = useSessionEvents(selectedRunId)

  const selectedRun = runs.find((run) => run.id === selectedRunId) ?? null
  const projects = React.useMemo(
    () =>
      toProjects(
        runs.map((run) => ({
          id: run.id,
          providerSessionId: run.providerSessionId,
          provider: run.provider,
          title: run.title,
          cwd: run.cwd,
          createdAt: run.createdAt.toISOString(),
          updatedAt: run.updatedAt.toISOString(),
          status: run.status,
          agentName: ACP_AGENT.name,
          agentVersion: ACP_AGENT.version,
        })),
      ),
    [runs],
  )

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

  const fetchAcp = React.useCallback(
    async (path: string, init?: RequestInit) => {
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
    },
    [acpBase, requestCredentials],
  )

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
    const payload = (await response.json()) as ProviderListPayload
    setProviderHealth(normalizeProviderHealth(payload))
  }, [fetchAcp])

  const createSession = React.useCallback<CreateSessionFn>(
    async (createOptions) => {
      const response = await fetchAcp('/sessions', {
        method: 'POST',
        body: JSON.stringify({}),
      })
      const payload = (await response.json()) as SessionCreatePayload
      const run = toRun(extractCreatedSession(payload))
      setRuns((current) => integrateCreatedRun(current, run))
      setSelectedRunId(run.id)
      if (createOptions?.closeSessionPanel) {
        onSessionPanelClose?.()
      }
      return run
    },
    [fetchAcp, onSessionPanelClose],
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
    if (!shouldAutoCreateInitialRun(Boolean(providerHealth?.ready), runs.length, selectedRunId)) {
      return
    }

    void (async () => {
      await createSessionForIntent(createSession, 'bootstrap', false)
    })()
  }, [createSession, providerHealth?.ready, runs.length, selectedRunId])

  const selectRun = React.useCallback(
    (runId: string) => {
      setSelectedRunId(runId)
      if (isMobileViewport) {
        onSessionPanelClose?.()
      }
    },
    [isMobileViewport, onSessionPanelClose],
  )

  const createRun = React.useCallback(async () => {
    await createSessionForIntent(createSession, 'manual', isMobileViewport)
  }, [createSession, isMobileViewport])

  const sendPrompt = React.useCallback(
    async (text: string) => {
      const runId = await ensurePromptRunId(selectedRunId, createSession, isMobileViewport)

      await fetchAcp(`/sessions/${runId}/prompt`, {
        method: 'POST',
        body: JSON.stringify({ prompt: text }),
      })
      await refreshSessions()
    },
    [createSession, fetchAcp, isMobileViewport, refreshSessions, selectedRunId],
  )

  return {
    runs,
    selectedRun,
    selectedRunId,
    providerHealth,
    selectedAgent,
    projects,
    events,
    messages,
    activity,
    connectionState,
    refreshSessions,
    refreshProvider,
    selectRun,
    createRun,
    sendPrompt,
  }
}
