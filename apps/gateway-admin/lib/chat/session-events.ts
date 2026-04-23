import type { BridgeEvent, BridgeSessionSummary } from '@/lib/acp/types'
import type { ACPMessage, TranscriptToolCall } from '@/components/chat/types'

export const MAX_SESSION_EVENTS = 500

function toDate(value: string) {
  return new Date(value)
}

export function appendSessionEvent(
  current: BridgeEvent[],
  incoming: BridgeEvent,
  maxEvents = MAX_SESSION_EVENTS,
) {
  const lastEvent = current.at(-1)
  if (lastEvent && incoming.seq <= lastEvent.seq) {
    return current
  }

  const next = [...current, incoming]
  if (next.length <= maxEvents) {
    return next
  }
  return next.slice(next.length - maxEvents)
}

export function resolveLastSessionEventSeq(events: BridgeEvent[], cachedLastSeq = 0) {
  return Math.max(cachedLastSeq, events.at(-1)?.seq ?? 0)
}

function upsertToolCall(toolCalls: TranscriptToolCall[], event: BridgeEvent): TranscriptToolCall[] {
  if (!event.toolCallId) {
    return toolCalls
  }

  const next = [...toolCalls]
  const index = next.findIndex((toolCall) => toolCall.id === event.toolCallId)
  const previous = index >= 0 ? next[index] : null
  const value: TranscriptToolCall = {
    id: event.toolCallId,
    title: event.title ?? previous?.title ?? event.toolCallId,
    status: event.status as TranscriptToolCall['status'],
    kind: event.toolKind ?? previous?.kind ?? null,
    input: event.rawInput ?? previous?.input,
    output: event.rawOutput ?? previous?.output,
    content: (event.toolContent as unknown[]) ?? previous?.content ?? null,
    locations: event.locations ?? previous?.locations ?? [],
  }

  if (index >= 0) {
    next[index] = value
  } else {
    next.push(value)
  }

  return next
}

function ensureAssistantMessage(
  sessionId: string,
  createdAt: string,
  messages: Map<string, ACPMessage>,
  orderedMessageIds: string[],
  preferredId: string | null,
) {
  const key = preferredId ?? `assistant-${sessionId}-${orderedMessageIds.length + 1}`
  let message = messages.get(key)

  if (!message) {
    message = {
      id: key,
      runId: sessionId,
      role: 'assistant',
      text: '',
      createdAt: toDate(createdAt),
      isStreaming: true,
      thoughts: [],
      toolCalls: [],
    }
    messages.set(key, message)
    orderedMessageIds.push(key)
  }

  return { key, message }
}

export function deriveTranscriptAndActivity(events: BridgeEvent[]): {
  messages: ACPMessage[]
  activity: BridgeEvent[]
} {
  const messages = new Map<string, ACPMessage>()
  const orderedMessageIds: string[] = []
  const activity: BridgeEvent[] = []
  let lastAssistantMessageId: string | null = null
  let activeUserMessageId: string | null = null
  let activeAssistantMessageId: string | null = null

  for (const event of events) {
    if (event.kind === 'message.chunk') {
      if (event.role === 'thinking') {
        activity.push(event)
        const { key, message } = ensureAssistantMessage(
          event.sessionId,
          event.createdAt,
          messages,
          orderedMessageIds,
          activeAssistantMessageId ?? lastAssistantMessageId,
        )
        if (event.text) {
          message.thoughts.push(event.text)
          message.createdAt = toDate(event.createdAt)
        }
        activeAssistantMessageId = key
        lastAssistantMessageId = key
        continue
      }

      const role = event.role === 'user' ? 'user' : 'assistant'
      const activeMessageId = role === 'user' ? activeUserMessageId : activeAssistantMessageId
      const key =
        event.messageId ??
        activeMessageId ??
        `${role}-${event.sessionId}-${orderedMessageIds.length + 1}`
      const existing = messages.get(key)

      if (!existing) {
        const message: ACPMessage = {
          id: key,
          runId: event.sessionId,
          role,
          text: event.text ?? '',
          createdAt: toDate(event.createdAt),
          isStreaming: role === 'assistant',
          thoughts: [],
          toolCalls: [],
        }
        messages.set(key, message)
        orderedMessageIds.push(key)
      } else {
        existing.text += event.text ?? ''
        existing.createdAt = toDate(event.createdAt)
      }

      if (role === 'user') {
        activeUserMessageId = key
        activeAssistantMessageId = null
      }

      if (role === 'assistant') {
        activeAssistantMessageId = key
        lastAssistantMessageId = key
      }
      continue
    }

    if (event.kind === 'tool.call' || event.kind === 'tool.update') {
      activity.push(event)
      const { key, message } = ensureAssistantMessage(
        event.sessionId,
        event.createdAt,
        messages,
        orderedMessageIds,
        activeAssistantMessageId ?? lastAssistantMessageId,
      )
      message.toolCalls = upsertToolCall(message.toolCalls, event)
      activeAssistantMessageId = key
      lastAssistantMessageId = key
      if (event.status === 'completed' || event.status === 'failed' || event.status === 'cancelled') {
        message.isStreaming = false
      }
      continue
    }

    if (event.kind === 'status') {
      const latest = lastAssistantMessageId ? messages.get(lastAssistantMessageId) : null
      if (latest) {
        latest.isStreaming = event.status === 'running'
      }
      if (event.status !== 'running') {
        activeAssistantMessageId = null
      }
    }

    activity.push(event)
  }

  return {
    messages: orderedMessageIds.map((id) => messages.get(id)!).filter(Boolean),
    activity,
  }
}

export function toProjects(sessions: BridgeSessionSummary[]) {
  if (sessions.length === 0) {
    return [{ id: 'workspace', name: 'workspace', agentId: 'codex' }]
  }

  const projectName = sessions[0]?.cwd.split('/').filter(Boolean).at(-1) ?? 'workspace'
  return [{ id: 'workspace', name: projectName, agentId: sessions[0]?.provider ?? 'codex' }]
}
