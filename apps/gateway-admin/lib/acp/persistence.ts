import path from 'node:path'
import { mkdir, readFile, rm, stat, writeFile } from 'node:fs/promises'
import type { BridgeEvent, BridgeSessionSummary } from './types'

export interface AcpSessionPersistence {
  loadSessions(): Promise<BridgeSessionSummary[]>
  loadEvents(sessionId: string): Promise<BridgeEvent[]>
  saveSession(session: BridgeSessionSummary): Promise<void>
  saveEvent(sessionId: string, event: BridgeEvent): Promise<void>
  deleteSession(sessionId: string): Promise<void>
}

export class NoopAcpSessionPersistence implements AcpSessionPersistence {
  async loadSessions(): Promise<BridgeSessionSummary[]> {
    return []
  }

  async loadEvents(_sessionId: string): Promise<BridgeEvent[]> {
    return []
  }

  async saveSession(_session: BridgeSessionSummary): Promise<void> {}
  async saveEvent(_sessionId: string, _event: BridgeEvent): Promise<void> {}
  async deleteSession(_sessionId: string): Promise<void> {}
}

type PersistedIndex = {
  sessions: BridgeSessionSummary[]
}

async function exists(filePath: string) {
  try {
    await stat(filePath)
    return true
  } catch {
    return false
  }
}

export class JsonFileAcpSessionPersistence implements AcpSessionPersistence {
  private readonly baseDir: string
  private readonly indexPath: string

  constructor(baseDir = path.resolve(process.cwd(), 'data', 'acp-sessions')) {
    this.baseDir = baseDir
    this.indexPath = path.join(this.baseDir, 'sessions.json')
  }

  async loadSessions(): Promise<BridgeSessionSummary[]> {
    await this.ensureBaseDir()
    if (!(await exists(this.indexPath))) {
      return []
    }

    const raw = await readFile(this.indexPath, 'utf8')
    const parsed = JSON.parse(raw) as PersistedIndex
    return parsed.sessions ?? []
  }

  async loadEvents(sessionId: string): Promise<BridgeEvent[]> {
    await this.ensureBaseDir()
    const filePath = this.eventPath(sessionId)

    if (!(await exists(filePath))) {
      return []
    }

    const raw = await readFile(filePath, 'utf8')
    if (!raw.trim()) {
      return []
    }

    return raw
      .split('\n')
      .filter(Boolean)
      .map((line) => JSON.parse(line) as BridgeEvent)
      .sort((left, right) => left.seq - right.seq)
  }

  async saveSession(session: BridgeSessionSummary): Promise<void> {
    await this.ensureBaseDir()
    const current = await this.loadSessions()
    const next = current.filter((item) => item.id !== session.id)
    next.push(session)
    next.sort((left, right) => right.updatedAt.localeCompare(left.updatedAt))
    await writeFile(this.indexPath, JSON.stringify({ sessions: next }, null, 2), 'utf8')
  }

  async saveEvent(sessionId: string, event: BridgeEvent): Promise<void> {
    await this.ensureBaseDir()
    const existing = await this.loadEvents(sessionId)
    if (existing.some((item) => item.id === event.id)) {
      return
    }

    existing.push(event)
    existing.sort((left, right) => left.seq - right.seq)
    const content = existing.map((item) => JSON.stringify(item)).join('\n')
    await writeFile(this.eventPath(sessionId), `${content}\n`, 'utf8')
  }

  async deleteSession(sessionId: string): Promise<void> {
    await this.ensureBaseDir()
    const current = await this.loadSessions()
    const next = current.filter((item) => item.id !== sessionId)
    await writeFile(this.indexPath, JSON.stringify({ sessions: next }, null, 2), 'utf8')
    await rm(this.eventPath(sessionId), { force: true })
  }

  private eventPath(sessionId: string) {
    return path.join(this.baseDir, `${sessionId}.jsonl`)
  }

  private async ensureBaseDir() {
    await mkdir(this.baseDir, { recursive: true })
  }
}
