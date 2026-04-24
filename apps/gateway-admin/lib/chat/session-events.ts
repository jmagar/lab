import type { AcpEvent, BridgeEvent, BridgeSessionSummary } from '@/lib/acp/types'
import type { ACPMessage, ActivityItem, TranscriptToolCall } from '@/components/chat/types'

export const MAX_SESSION_EVENTS = 500

function toDate(value: string) {
  return new Date(value)
}

type ToolCallPatch = {
  id: string
  title?: string
  status?: TranscriptToolCall['status']
  kind?: string | null
  input?: unknown
  output?: unknown
  content?: unknown[] | null
  locations?: string[]
}

const INTERNAL_EVENT_KINDS = new Set(['tool_call_metadata'])

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value)
}

function isToolCallStatus(value: unknown): value is TranscriptToolCall['status'] | undefined {
  switch (value) {
    case 'pending':
    case 'in_progress':
    case 'completed':
    case 'failed':
    case 'idle':
    case 'running':
    case 'cancelled':
      return value
    default:
      return undefined
  }
}

function readToolMetadata(raw: unknown) {
  if (!isRecord(raw)) {
    return null
  }

  const hasMetadataShape =
    'tool_kind' in raw ||
    'kind' in raw ||
    'locations' in raw ||
    'content' in raw ||
    'raw_output' in raw

  if (!hasMetadataShape) {
    return null
  }

  return {
    title: typeof raw.title === 'string' ? raw.title : undefined,
    kind: typeof raw.tool_kind === 'string' ? raw.tool_kind : typeof raw.kind === 'string' ? raw.kind : null,
    status: isToolCallStatus(raw.status),
    content: Array.isArray(raw.content) ? raw.content : undefined,
    locations: Array.isArray(raw.locations)
      ? raw.locations.filter((value): value is string => typeof value === 'string')
      : undefined,
    output: raw.raw_output,
  }
}

function providerRawType(raw: unknown) {
  if (!isRecord(raw) || typeof raw.type !== 'string') {
    return null
  }
  return raw.type
}

function bridgeBaseEvent(event: AcpEvent, kind: BridgeEvent['kind'], provider = 'codex'): BridgeEvent {
  return {
    id: event.id,
    seq: event.seq,
    sessionId: event.session_id,
    provider,
    kind,
    createdAt: event.created_at,
  }
}

export function bridgeEventFromAcpEvent(event: AcpEvent): BridgeEvent {
  switch (event.kind) {
    case 'message_chunk':
      return {
        ...bridgeBaseEvent(event, 'message.chunk'),
        role: event.role,
        text: event.text,
        messageId: event.message_id,
      }
    case 'reasoning_chunk':
      return {
        ...bridgeBaseEvent(event, 'message.chunk'),
        role: 'thinking',
        text: event.text,
      }
    case 'tool_call_start':
      return {
        ...bridgeBaseEvent(event, 'tool.call'),
        title: event.name,
        toolCallId: event.tool_call_id,
        rawInput: event.input,
      }
    case 'tool_call_update':
      return {
        ...bridgeBaseEvent(event, 'tool.update'),
        title: 'Tool call updated',
        toolCallId: event.tool_call_id,
        status: event.status,
        rawOutput: event.output,
      }
    case 'permission_request':
      return {
        ...bridgeBaseEvent(event, 'permission.requested'),
        title: event.action_summary,
        toolCallId: event.request_id,
        status: 'requested',
        permissionOptions: event.options.map((option) => ({
          optionId: option.option_id,
          name: option.name,
          kind: option.kind,
        })),
      }
    case 'permission_outcome':
      return {
        ...bridgeBaseEvent(event, 'permission.resolved'),
        toolCallId: event.request_id,
        status: 'resolved',
        permissionSelection: event.granted ? 'granted' : 'rejected',
      }
    case 'usage_update':
      return {
        ...bridgeBaseEvent(event, 'usage'),
        usage: event.raw as BridgeEvent['usage'],
      }
    case 'content_blocks':
      return {
        ...bridgeBaseEvent(event, 'content'),
        raw: { blocks: event.blocks },
      }
    case 'session_update':
      return {
        ...bridgeBaseEvent(event, 'status'),
        status: event.state,
      }
    case 'provider_info': {
      const rawType = providerRawType(event.raw)
      switch (rawType) {
        case 'stderr':
        case 'debug':
          return {
            ...bridgeBaseEvent(event, 'debug', event.provider),
            title: isRecord(event.raw) && typeof event.raw.title === 'string' ? event.raw.title : undefined,
            text: isRecord(event.raw) && typeof event.raw.text === 'string' ? event.raw.text : undefined,
            raw: event.raw,
          }
        case 'plan':
          return {
            ...bridgeBaseEvent(event, 'plan', event.provider),
            title: isRecord(event.raw) && typeof event.raw.title === 'string' ? event.raw.title : undefined,
            plan:
              isRecord(event.raw) && Array.isArray(event.raw.entries)
                ? (event.raw.entries as BridgeEvent['plan'])
                : undefined,
            raw: event.raw,
          }
        case 'commands':
          return {
            ...bridgeBaseEvent(event, 'commands', event.provider),
            title: isRecord(event.raw) && typeof event.raw.title === 'string' ? event.raw.title : undefined,
            commands:
              isRecord(event.raw) && Array.isArray(event.raw.commands)
                ? (event.raw.commands as BridgeEvent['commands'])
                : undefined,
            raw: event.raw,
          }
        case 'current_mode':
          return {
            ...bridgeBaseEvent(event, 'mode', event.provider),
            title: isRecord(event.raw) && typeof event.raw.title === 'string' ? event.raw.title : undefined,
            currentMode: isRecord(event.raw) ? (event.raw.current_mode as BridgeEvent['currentMode']) : undefined,
            raw: event.raw,
          }
        case 'config_update':
          return {
            ...bridgeBaseEvent(event, 'config', event.provider),
            title: isRecord(event.raw) && typeof event.raw.title === 'string' ? event.raw.title : undefined,
            configUpdate: isRecord(event.raw) ? (event.raw.config_update as BridgeEvent['configUpdate']) : undefined,
            raw: event.raw,
          }
        case 'session_info':
          return {
            ...bridgeBaseEvent(event, 'session.info', event.provider),
            title:
              isRecord(event.raw) && isRecord(event.raw.session_info) && typeof event.raw.session_info.title === 'string'
                ? event.raw.session_info.title
                : isRecord(event.raw) && typeof event.raw.title === 'string'
                  ? event.raw.title
                  : undefined,
            sessionInfo: isRecord(event.raw) ? (event.raw.session_info as BridgeEvent['sessionInfo']) : undefined,
            raw: event.raw,
          }
        case 'stop_reason':
          return {
            ...bridgeBaseEvent(event, 'status', event.provider),
            title: isRecord(event.raw) && typeof event.raw.title === 'string' ? event.raw.title : undefined,
            status: isRecord(event.raw) && typeof event.raw.status === 'string' ? event.raw.status : undefined,
            promptStopReason:
              isRecord(event.raw) && typeof event.raw.stop_reason === 'string'
                ? (event.raw.stop_reason as BridgeEvent['promptStopReason'])
                : undefined,
            raw: event.raw,
          }
        case 'subscriber_backpressure':
          return {
            ...bridgeBaseEvent(event, 'reconnect', event.provider),
            title: 'Subscriber backpressure',
            status: 'reconnect',
            raw: event.raw,
          }
        default:
          return {
            ...bridgeBaseEvent(event, rawType ?? 'debug', event.provider),
            raw: event.raw,
          }
      }
    }
    case 'unknown':
      return {
        ...bridgeBaseEvent(event, event.event_kind),
        raw: event.raw,
      }
  }
}

function toolPatchFromEvent(event: BridgeEvent): ToolCallPatch | null {
  if (event.kind === 'tool.call') {
    if (!event.toolCallId) {
      return null
    }

    return {
      id: event.toolCallId,
      title: event.title,
      status: isToolCallStatus(event.status),
      kind: event.toolKind ?? undefined,
      input: event.rawInput,
      output: event.rawOutput,
      content: (event.toolContent as unknown[]) ?? undefined,
      locations: event.locations,
    }
  }

  if (event.kind === 'tool.update') {
    if (!event.toolCallId) {
      return null
    }

    const outputMetadata = readToolMetadata(event.rawOutput)
    return {
      id: event.toolCallId,
      title: outputMetadata?.title,
      status: isToolCallStatus(event.status) ?? outputMetadata?.status,
      kind: outputMetadata?.kind,
      output: event.rawOutput,
      content: outputMetadata?.content,
      locations: outputMetadata?.locations,
    }
  }

  if (event.kind === 'tool_call_metadata') {
    const metadata = readToolMetadata(event.raw)
    const toolCallId =
      event.toolCallId ??
      (isRecord(event.raw) && typeof event.raw.tool_call_id === 'string' ? event.raw.tool_call_id : null)

    if (!toolCallId || !metadata) {
      return null
    }

    return {
      id: toolCallId,
      title: metadata.title ?? event.title,
      status: metadata.status,
      kind: metadata.kind,
      content: metadata.content,
      locations: metadata.locations,
      output: metadata.output,
    }
  }

  return null
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

function upsertToolCall(toolCalls: TranscriptToolCall[], patch: ToolCallPatch): TranscriptToolCall[] {
  const next = [...toolCalls]
  const index = next.findIndex((toolCall) => toolCall.id === patch.id)
  const previous = index >= 0 ? next[index] : null
  const value: TranscriptToolCall = {
    id: patch.id,
    title: patch.title ?? previous?.title ?? patch.id,
    status: patch.status ?? previous?.status,
    kind: patch.kind ?? previous?.kind ?? null,
    input: patch.input ?? previous?.input,
    output: patch.output ?? previous?.output,
    content: patch.content ?? previous?.content ?? null,
    locations: patch.locations ?? previous?.locations ?? [],
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
  activity: ActivityItem[]
} {
  const messages = new Map<string, ACPMessage>()
  const orderedMessageIds: string[] = []
  const activity: ActivityItem[] = []
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

    const toolPatch = toolPatchFromEvent(event)
    if (toolPatch) {
      if (!INTERNAL_EVENT_KINDS.has(event.kind)) {
        activity.push(event)
      }
      const { key, message } = ensureAssistantMessage(
        event.sessionId,
        event.createdAt,
        messages,
        orderedMessageIds,
        activeAssistantMessageId ?? lastAssistantMessageId,
      )
      message.toolCalls = upsertToolCall(message.toolCalls, toolPatch)
      activeAssistantMessageId = key
      lastAssistantMessageId = key
      if (
        toolPatch.status === 'completed' ||
        toolPatch.status === 'failed' ||
        toolPatch.status === 'cancelled'
      ) {
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
