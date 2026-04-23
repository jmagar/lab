import path from 'node:path'
import type { AcpProvider, AcpProviderHandle } from './provider'
import {
  JsonFileAcpSessionPersistence,
  type AcpSessionPersistence,
} from './persistence'
import { normalizeProviderEvent } from './normalize'
import { CodexAcpProvider, attachProviderEventHandler } from './providers/codex-acp'
import type {
  BridgeEvent,
  BridgeSessionSummary,
  ProviderHealth,
  StartSessionInput,
  StartSessionResult,
} from './types'

type ActiveSessionRecord = {
  summary: BridgeSessionSummary
  handle: AcpProviderHandle | null
  events: BridgeEvent[]
  nextSeq: number
  subscribers: Set<(event: BridgeEvent) => void>
}

export class AcpSessionRegistry {
  private readonly sessions = new Map<string, ActiveSessionRecord>()
  private readonly provider: AcpProvider
  private readonly persistence: AcpSessionPersistence
  private readonly defaultCwd: string
  private readonly hydration: Promise<void>

  constructor(options?: {
    provider?: AcpProvider
    persistence?: AcpSessionPersistence
    defaultCwd?: string
  }) {
    this.provider = options?.provider ?? new CodexAcpProvider()
    this.persistence = options?.persistence ?? new JsonFileAcpSessionPersistence()
    this.defaultCwd =
      options?.defaultCwd ??
      process.env.ACP_SESSION_CWD ??
      path.resolve(process.cwd(), '..', '..')
    this.hydration = this.hydrate()
  }

  async health(): Promise<ProviderHealth> {
    return this.provider.health()
  }

  async listSessions(): Promise<BridgeSessionSummary[]> {
    await this.ensureHydrated()
    return Array.from(this.sessions.values())
      .map((record) => record.summary)
      .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt))
  }

  async getSession(sessionId: string): Promise<BridgeSessionSummary | null> {
    await this.ensureHydrated()
    return this.sessions.get(sessionId)?.summary ?? null
  }

  async createSession(input?: Partial<StartSessionInput>): Promise<BridgeSessionSummary> {
    await this.ensureHydrated()

    const resolvedInput: StartSessionInput = {
      cwd: input?.cwd ?? this.defaultCwd,
      title: input?.title,
      clientCapabilities: input?.clientCapabilities,
    }

    const uiSessionId = crypto.randomUUID()
    const createdAt = new Date().toISOString()
    const { handle, result } = await this.startProviderSession(uiSessionId, resolvedInput)

    const summary: BridgeSessionSummary = {
      id: uiSessionId,
      providerSessionId: result.providerSessionId,
      provider: this.provider.kind,
      title: resolvedInput.title ?? 'New session',
      cwd: resolvedInput.cwd,
      createdAt,
      updatedAt: createdAt,
      status: 'idle',
      agentName: result.agentName,
      agentVersion: result.agentVersion,
      resumable: true,
    }

    this.sessions.set(uiSessionId, {
      summary,
      handle,
      events: [],
      nextSeq: 1,
      subscribers: new Set(),
    })

    await this.persistence.saveSession(summary)
    return summary
  }

  async promptSession(sessionId: string, prompt: string): Promise<void> {
    await this.ensureHydrated()
    const record = this.requireSession(sessionId)

    if (!record.handle) {
      await this.reattachSession(record)
    }

    record.summary = {
      ...record.summary,
      status: 'running',
      updatedAt: new Date().toISOString(),
      resumable: true,
    }
    await this.persistence.saveSession(record.summary)
    await this.provider.promptSession(record.handle!, prompt)
  }

  async cancelSession(sessionId: string): Promise<void> {
    await this.ensureHydrated()
    const record = this.requireSession(sessionId)
    record.summary = {
      ...record.summary,
      status: 'cancelled',
      updatedAt: new Date().toISOString(),
    }
    await this.persistence.saveSession(record.summary)
    if (record.handle) {
      await this.provider.cancelSession(record.handle)
    }
  }

  async subscribe(
    sessionId: string,
    sinceSeq: number,
    onEvent: (event: BridgeEvent) => void,
  ): Promise<() => void> {
    await this.ensureHydrated()
    const record = this.requireSession(sessionId)
    for (const event of record.events) {
      if (event.seq > sinceSeq) {
        onEvent(event)
      }
    }
    record.subscribers.add(onEvent)
    return () => {
      record.subscribers.delete(onEvent)
    }
  }

  private async hydrate() {
    const sessions = await this.persistence.loadSessions()
    for (const summary of sessions) {
      const events = await this.persistence.loadEvents(summary.id)
      this.sessions.set(summary.id, {
        summary: { ...summary, resumable: true },
        handle: null,
        events,
        nextSeq: (events.at(-1)?.seq ?? 0) + 1,
        subscribers: new Set(),
      })
    }
  }

  private async ensureHydrated() {
    await this.hydration
  }

  private requireSession(sessionId: string): ActiveSessionRecord {
    const record = this.sessions.get(sessionId)
    if (!record) {
      throw new Error(`Unknown session: ${sessionId}`)
    }
    return record
  }

  private async reattachSession(record: ActiveSessionRecord) {
    const { handle, result } = await this.startProviderSession(record.summary.id, {
      cwd: record.summary.cwd,
      title: record.summary.title,
    })

    record.handle = handle
    record.summary = {
      ...record.summary,
      providerSessionId: result.providerSessionId,
      provider: this.provider.kind,
      agentName: result.agentName,
      agentVersion: result.agentVersion,
      updatedAt: new Date().toISOString(),
      resumable: true,
    }

    this.pushEvent(record.summary.id, {
      sessionId: record.summary.id,
      provider: this.provider.kind,
      kind: 'status',
      createdAt: new Date().toISOString(),
      title: 'Session runtime recreated',
      text: 'The provider process was restarted and this session was reattached with a new runtime.',
      status: 'idle',
    })
  }

  private async startProviderSession(
    sessionId: string,
    input: StartSessionInput,
  ): Promise<{ handle: AcpProviderHandle; result: StartSessionResult }> {
    const started = await this.provider.startSession(input, (rawEvent) => {
      const normalizedEvents = normalizeProviderEvent(sessionId, this.provider.kind, rawEvent)
      for (const partialEvent of normalizedEvents) {
        this.pushEvent(sessionId, partialEvent)
      }
    })

    attachProviderEventHandler(started.handle, (rawEvent) => {
      const normalizedEvents = normalizeProviderEvent(sessionId, this.provider.kind, rawEvent)
      for (const partialEvent of normalizedEvents) {
        this.pushEvent(sessionId, partialEvent)
      }
    })

    return started
  }

  private pushEvent(sessionId: string, partialEvent: Omit<BridgeEvent, 'id' | 'seq'>) {
    const record = this.sessions.get(sessionId)
    if (!record) {
      return
    }

    const event: BridgeEvent = {
      ...partialEvent,
      id: crypto.randomUUID(),
      seq: record.nextSeq++,
    }

    record.events.push(event)
    if (record.events.length > 500) {
      record.events.splice(0, record.events.length - 500)
    }

    record.summary = this.applyEventToSummary(record.summary, event)
    void this.persistence.saveEvent(sessionId, event)
    void this.persistence.saveSession(record.summary)

    for (const subscriber of record.subscribers) {
      subscriber(event)
    }
  }

  private applyEventToSummary(summary: BridgeSessionSummary, event: BridgeEvent): BridgeSessionSummary {
    const updatedAt = event.createdAt

    if (event.kind === 'session.info' && event.sessionInfo?.title) {
      return {
        ...summary,
        title: event.sessionInfo.title,
        updatedAt,
      }
    }

    if (event.kind === 'status') {
      return {
        ...summary,
        status: (event.status as BridgeSessionSummary['status']) ?? summary.status,
        updatedAt,
      }
    }

    if (event.kind === 'error') {
      return {
        ...summary,
        status: 'failed',
        updatedAt,
      }
    }

    return {
      ...summary,
      updatedAt,
    }
  }
}

declare global {
  var __gatewayAdminAcpRegistry: AcpSessionRegistry | undefined
}

export function getAcpSessionRegistry() {
  if (!globalThis.__gatewayAdminAcpRegistry) {
    globalThis.__gatewayAdminAcpRegistry = new AcpSessionRegistry()
  }
  return globalThis.__gatewayAdminAcpRegistry
}
