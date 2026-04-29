'use client'

import * as React from 'react'
import { createAcpFetcher } from '@/lib/acp/fetch'
import { toProjects } from './session-events'
import { useSessionEvents } from './use-session-events'
import {
  type ProviderListPayload,
  type SessionCreatePayload,
  type ErrorPayload,
  type RawSessionSummary,
  toRun,
  normalizeProviderHealth,
  normalizeProviderList,
  extractCreatedSession,
  readJsonSafe,
  errorMessageFromPayload,
} from './acp-normalizers'
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

export function useChatSessionController(options: {
  isMobileViewport: boolean
  onSessionPanelClose?: () => void
}) {
  const { isMobileViewport, onSessionPanelClose } = options
  const fetchAcpRef = React.useRef(createAcpFetcher())
  const fetchAcp = React.useCallback(
    (path: string, init?: RequestInit) => fetchAcpRef.current(path, init),
    [],
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

  const refreshSessions = React.useCallback(async () => {
    const response = await fetchAcp('/sessions')
    if (!response.ok) {
      // On error: preserve current selectedRunId — resetting it to null would
      // re-trigger the shouldAutoCreateInitialRun bootstrap loop.
      setRuns([])
      return
    }
    const payload = await readJsonSafe<{ sessions: RawSessionSummary[] }>(response)
    if (!payload) {
      // Same: keep selectedRunId stable on parse error.
      setRuns([])
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
      // Narrow: SessionCreatePayload has an `id` field; ErrorPayload does not
      const isSessionPayload = 'id' in payload || ('session' in payload && payload.session != null)
      if (!isSessionPayload) {
        const message = errorMessageFromPayload(payload as ErrorPayload, 'Failed to create ACP session.')
        throw new Error(message)
      }
      const run = toRun(extractCreatedSession(payload as SessionCreatePayload))
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

  // Keep a stable ref so the bootstrap effect does not re-fire every time
  // createSession's identity changes (which happens on every providerHealth update).
  const createSessionBootstrapRef = React.useRef(createSession)
  React.useEffect(() => {
    createSessionBootstrapRef.current = createSession
  })

  React.useEffect(() => {
    if (!shouldAutoCreateInitialRun(Boolean(providerHealth?.ready), runs.length, selectedRunId)) {
      return
    }

    void (async () => {
      await createSessionForIntent(createSessionBootstrapRef.current, 'bootstrap', false)
    })()
  // Intentionally omit createSession — use ref above to avoid re-firing on identity changes.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [providerHealth?.ready, runs.length, selectedRunId])

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
