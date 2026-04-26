'use client'

import * as React from 'react'
import { normalizeGatewayApiBase } from '@/lib/api/gateway-config'
import { gatewayHeaders } from '@/lib/api/gateway-request'
import { isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'
import { toProjects } from './session-events'
import { useSessionEvents } from './use-session-events'
import type { ACPAgent, ACPRun } from '@/components/chat/types'
import type { BridgeSessionSummary, ProviderHealth } from '@/lib/acp/types'
import type { AttachmentRef } from '@/lib/fs/types'

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
    command?: string | null
    args?: string[] | null
  }>
  provider?: ProviderHealth
}

type SessionCreatePayload = {
  session?: BridgeSessionSummary
} & BridgeSessionSummary

type ErrorPayload = {
  message?: string
  error?: string
  kind?: string
}

type RawSessionSummary = {
  id: string
  provider: string
  title: string
  cwd: string
  status?: string
  state?: string
  createdAt?: string
  updatedAt?: string
  providerSessionId?: string
  agentName?: string
  agentVersion?: string
  created_at?: string
  updated_at?: string
  provider_session_id?: string
  agent_name?: string
  agent_version?: string
}

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

function normalizeSessionSummary(session: RawSessionSummary): BridgeSessionSummary {
  return {
    id: session.id,
    provider: session.provider,
    title: session.title,
    cwd: session.cwd,
    status: (session.status ?? session.state ?? 'idle') as BridgeSessionSummary['status'],
    createdAt: session.createdAt ?? session.created_at ?? '',
    updatedAt: session.updatedAt ?? session.updated_at ?? '',
    providerSessionId: session.providerSessionId ?? session.provider_session_id ?? '',
    agentName: session.agentName ?? session.agent_name ?? 'Codex',
    agentVersion: session.agentVersion ?? session.agent_version ?? 'unknown',
  }
}

function toRun(session: RawSessionSummary): ACPRun {
  const normalized = normalizeSessionSummary(session)
  return {
    id: normalized.id,
    projectId: 'workspace',
    agentId: normalized.provider,
    provider: normalized.provider,
    title: normalized.title,
    createdAt: new Date(normalized.createdAt),
    updatedAt: new Date(normalized.updatedAt),
    status: normalized.status,
    providerSessionId: normalized.providerSessionId,
    cwd: normalized.cwd,
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

function normalizeProviderList(payload: ProviderListPayload): ProviderHealth[] {
  if (payload.provider) {
    return [payload.provider]
  }

  return (payload.providers ?? []).map((provider) => ({
    provider: provider.name ?? 'codex-acp',
    ready: Boolean(provider.available),
    command: provider.command ?? '',
    args: provider.args ?? [],
    message: provider.error ?? '',
  }))
}

function extractCreatedSession(payload: SessionCreatePayload): RawSessionSummary {
  return payload.session ?? payload
}

async function readJsonSafe<T>(response: Response): Promise<T | null> {
  const text = await response.text()
  if (!text) {
    return null
  }

  try {
    return JSON.parse(text) as T
  } catch {
    return null
  }
}

function errorMessageFromPayload(payload: ErrorPayload | null, fallback: string) {
  return payload?.message ?? payload?.error ?? fallback
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
  const [providers, setProviders] = React.useState<ProviderHealth[]>([])
  const [selectedProviderId, setSelectedProviderId] = React.useState<string | null>(null)
  const { events, messages, activity, connectionState, sessionStatus } = useSessionEvents(selectedRunId)

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

  const agents = React.useMemo<ACPAgent[]>(
    () =>
      providers.map((provider) => ({
        ...ACP_AGENT,
        id: provider.provider,
        name: provider.provider === 'codex-acp' ? 'Codex ACP' : provider.provider,
        version: ACP_AGENT.version,
      })),
    [providers],
  )

  const selectedAgent = selectedRun
    ? (agents.find((agent) => agent.id === selectedRun.provider) ?? {
        ...ACP_AGENT,
        id: selectedRun.provider,
        name: selectedRun.provider,
      })
    : (agents.find((agent) => agent.id === selectedProviderId) ?? agents[0] ?? ACP_AGENT)

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
    const payload = await readJsonSafe<{ sessions: RawSessionSummary[] }>(response)
    if (!payload) {
      setRuns([])
      setSelectedRunId(null)
      return
    }
    const nextRuns = payload.sessions.map(toRun)
    setRuns(nextRuns)
    setSelectedRunId((current) => current ?? nextRuns[0]?.id ?? null)
  }, [fetchAcp])

  const refreshProvider = React.useCallback(async () => {
    const response = await fetchAcp('/provider')
    if (!response.ok) {
      const unavailable = {
        provider: 'codex-acp',
        ready: false,
        command: '',
        args: [],
        message: 'ACP provider unavailable.',
      }
      setProviders([unavailable])
      setSelectedProviderId('codex-acp')
      setProviderHealth(unavailable)
      return
    }
    const payload = (await response.json()) as ProviderListPayload
    const nextProviders = normalizeProviderList(payload)
    const selected = nextProviders.find((provider) => provider.ready) ?? nextProviders[0] ?? normalizeProviderHealth(payload)
    setProviders(nextProviders.length > 0 ? nextProviders : [selected])
    setSelectedProviderId((current) =>
      current && nextProviders.some((provider) => provider.provider === current)
        ? current
        : selected.provider,
    )
    setProviderHealth(selected)
  }, [fetchAcp])

  const createSession = React.useCallback<CreateSessionFn>(
    async (createOptions) => {
      const response = await fetchAcp('/sessions', {
        method: 'POST',
        body: JSON.stringify({ provider: selectedProviderId ?? providerHealth?.provider ?? 'codex-acp' }),
      })
      const payload = await readJsonSafe<SessionCreatePayload | ErrorPayload>(response)
      if (!response.ok || !payload) {
        const message = errorMessageFromPayload(
          payload as ErrorPayload | null,
          'Failed to create ACP session.',
        )
        setProviderHealth((current) => ({
          provider: current?.provider ?? 'codex-acp',
          ready: false,
          command: current?.command ?? '',
          args: current?.args ?? [],
          message,
        }))
        throw new Error(message)
      }
      const run = toRun(extractCreatedSession(payload))
      setRuns((current) => integrateCreatedRun(current, run))
      setSelectedRunId(run.id)
      if (createOptions?.closeSessionPanel) {
        onSessionPanelClose?.()
      }
      return run
    },
    [fetchAcp, onSessionPanelClose, providerHealth?.provider, selectedProviderId],
  )

  React.useEffect(() => {
    if (USE_MOCK_DATA) {
      setProviderHealth({
        provider: 'codex-acp',
        ready: false,
        command: '',
        args: [],
        message: 'ACP unavailable in mock preview.',
      })
      setProviders([])
      setSelectedProviderId(null)
      setRuns([])
      setSelectedRunId(null)
      return
    }

    void refreshProvider()
    void refreshSessions()
  }, [refreshProvider, refreshSessions])

  React.useEffect(() => {
    if (!selectedProviderId) {
      return
    }
    const selected = providers.find((provider) => provider.provider === selectedProviderId)
    if (selected) {
      setProviderHealth(selected)
    }
  }, [providers, selectedProviderId])

  React.useEffect(() => {
    if (!shouldAutoCreateInitialRun(Boolean(providerHealth?.ready), runs.length, selectedRunId)) {
      return
    }

    void (async () => {
      await createSessionForIntent(createSession, 'bootstrap', false)
    })()
  }, [createSession, providerHealth?.ready, runs.length, selectedRunId])

  React.useEffect(() => {
    if (!selectedRunId || events.length === 0) {
      return
    }

    setRuns((current) =>
      current.map((run) =>
        run.id === selectedRunId && run.status !== sessionStatus
          ? { ...run, status: sessionStatus, updatedAt: new Date() }
          : run,
      ),
    )
  }, [events.length, selectedRunId, sessionStatus])

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
    try {
      await createSessionForIntent(createSession, 'manual', isMobileViewport)
    } catch {
      // keep the UI responsive; providerHealth.message carries the failure detail
    }
  }, [createSession, isMobileViewport])

  const sendPrompt = React.useCallback(
    async (payload: { text: string; attachments: AttachmentRef[] }) => {
      try {
        const runId = await ensurePromptRunId(selectedRunId, createSession, isMobileViewport)

        const body = {
          prompt: payload.text,
          ...(payload.attachments.length > 0 && { attachments: payload.attachments }),
        }

        const response = await fetchAcp(`/sessions/${runId}/prompt`, {
          method: 'POST',
          body: JSON.stringify(body),
        })
        if (!response.ok) {
          const errorPayload = await readJsonSafe<ErrorPayload>(response)
          const message = errorMessageFromPayload(
            errorPayload,
            'Failed to send prompt to ACP session.',
          )
          setProviderHealth((current) => ({
        provider: current?.provider ?? 'codex-acp',
            ready: false,
            command: current?.command ?? '',
            args: current?.args ?? [],
            message,
          }))
          return
        }

        await refreshSessions()
      } catch {
        // keep the UI responsive; providerHealth.message carries the failure detail
      }
    },
    [createSession, fetchAcp, isMobileViewport, refreshSessions, selectedRunId],
  )

  return {
    runs,
    selectedRun,
    selectedRunId,
    providerHealth,
    selectedAgent,
    agents,
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
    selectAgent: setSelectedProviderId,
  }
}
