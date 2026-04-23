import type { BridgeEvent, BridgeSessionSummary } from '@/lib/acp/types'
import type { ACPMessage, TranscriptToolCall } from '@/components/chat/types'

function toDate(value: string) {
  return new Date(value)
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
    locations: event.locations ?? previous?.locations ?? [],
  }

  if (index >= 0) {
    next[index] = value
  } else {
    next.push(value)
  }

  return next
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
        if (lastAssistantMessageId) {
          const message = messages.get(lastAssistantMessageId)
          if (message && event.text) {
            message.thoughts.push(event.text)
            message.createdAt = toDate(event.createdAt)
          }
        }
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
      if (lastAssistantMessageId) {
        const message = messages.get(lastAssistantMessageId)
        if (message) {
          message.toolCalls = upsertToolCall(message.toolCalls, event)
        }
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
