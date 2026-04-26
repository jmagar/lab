import type {
  AvailableCommandsUpdate,
  ConfigOptionUpdate,
  CurrentModeUpdate,
  Plan,
  SessionInfoUpdate,
  ToolCall,
  ToolCallUpdate,
  UsageUpdate,
} from '@agentclientprotocol/sdk'
import type { AcpProviderKind, BridgeEvent, ProviderRuntimeEvent } from './types'

type PendingBridgeEvent = Omit<BridgeEvent, 'id' | 'seq'>

function baseEvent(
  sessionId: string,
  provider: AcpProviderKind,
  kind: BridgeEvent['kind'],
): PendingBridgeEvent {
  return {
    sessionId,
    provider,
    kind,
    createdAt: new Date().toISOString(),
  }
}

function contentToText(content: unknown): string {
  if (!content || typeof content !== 'object') {
    return ''
  }

  const block = content as { type?: string; text?: string; uri?: string; mimeType?: string }

  if (block.type === 'text' && typeof block.text === 'string') {
    return block.text
  }

  if (block.type === 'image') {
    return '[image]'
  }

  if (block.type === 'audio') {
    return '[audio]'
  }

  if (block.type === 'resource_link' && typeof block.uri === 'string') {
    return `[resource] ${block.uri}`
  }

  if (block.type === 'resource') {
    return '[embedded resource]'
  }

  return '[content]'
}

function toolCallMetadataEvent(
  sessionId: string,
  provider: AcpProviderKind,
  toolCallId: string,
  payload: {
    title?: string
    toolKind?: ToolCall['kind'] | ToolCallUpdate['kind'] | null
    status?: ToolCall['status'] | ToolCallUpdate['status']
    locations?: string[]
    content?: ToolCall['content'] | ToolCallUpdate['content'] | null
    rawOutput?: unknown
  },
): PendingBridgeEvent | null {
  const locations = payload.locations ?? []
  const hasPayload =
    payload.title !== undefined ||
    payload.toolKind !== undefined ||
    payload.status !== undefined ||
    locations.length > 0 ||
    payload.content !== undefined ||
    payload.rawOutput !== undefined

  if (!hasPayload) {
    return null
  }

  return {
    ...baseEvent(sessionId, provider, 'tool_call_metadata'),
    toolCallId,
    title: payload.title,
    raw: {
      type: 'tool_call_metadata',
      tool_call_id: toolCallId,
      title: payload.title,
      tool_kind: payload.toolKind ?? null,
      status: payload.status,
      locations,
      content: payload.content ?? null,
      raw_output: payload.rawOutput,
    },
  }
}

function normalizeToolCall(
  sessionId: string,
  provider: AcpProviderKind,
  update: ToolCall,
): PendingBridgeEvent {
  return {
    ...baseEvent(sessionId, provider, 'tool.call'),
    title: update.title,
    toolCallId: update.toolCallId,
    rawInput: update.rawInput,
  }
}

function toolCallUpdateOutput(update: ToolCallUpdate): unknown {
  if (update.rawOutput !== undefined) {
    return update.rawOutput
  }

  const output: Record<string, unknown> = {}

  if (update.title !== undefined) {
    output.title = update.title
  }
  if (update.kind !== undefined) {
    output.kind = update.kind
  }
  if (update.status !== undefined) {
    output.status = update.status
  }
  if (update.content !== undefined) {
    output.content = update.content
  }
  if (update.locations !== undefined) {
    output.locations = update.locations.map((location) => location.path)
  }

  return Object.keys(output).length > 0 ? output : null
}

function normalizeToolUpdate(
  sessionId: string,
  provider: AcpProviderKind,
  update: ToolCallUpdate,
): PendingBridgeEvent {
  return {
    ...baseEvent(sessionId, provider, 'tool.update'),
    title: 'Tool call updated',
    toolCallId: update.toolCallId,
    status: update.status ?? 'updated',
    rawOutput: toolCallUpdateOutput(update),
  }
}

export function normalizeProviderEvent(
  sessionId: string,
  provider: AcpProviderKind,
  event: ProviderRuntimeEvent,
): PendingBridgeEvent[] {
  switch (event.type) {
    case 'session_notification': {
      const update = event.notification.update
      switch (update.sessionUpdate) {
        case 'user_message_chunk':
          return [
            {
              ...baseEvent(sessionId, provider, 'message.chunk'),
              role: 'user',
              messageId: update.messageId ?? null,
              text: contentToText(update.content),
            },
          ]
        case 'agent_message_chunk':
          return [
            {
              ...baseEvent(sessionId, provider, 'message.chunk'),
              role: 'assistant',
              messageId: update.messageId ?? null,
              text: contentToText(update.content),
            },
          ]
        case 'agent_thought_chunk':
          return [
            {
              ...baseEvent(sessionId, provider, 'message.chunk'),
              role: 'thinking',
              messageId: update.messageId ?? null,
              text: contentToText(update.content),
            },
          ]
        case 'tool_call':
          return [
            normalizeToolCall(sessionId, provider, update),
            toolCallMetadataEvent(sessionId, provider, update.toolCallId, {
              title: update.title,
              toolKind: update.kind,
              status: update.status,
              locations: update.locations?.map((location) => location.path),
              content: update.content,
              rawOutput: update.rawOutput,
            }),
          ].filter((pending): pending is PendingBridgeEvent => pending !== null)
        case 'tool_call_update':
          return [normalizeToolUpdate(sessionId, provider, update)]
        case 'plan':
          return [
            {
              ...baseEvent(sessionId, provider, 'plan'),
              plan: (update as Plan).entries,
              title: 'Execution plan updated',
            },
          ]
        case 'available_commands_update':
          return [
            {
              ...baseEvent(sessionId, provider, 'commands'),
              commands: (update as AvailableCommandsUpdate).availableCommands,
              title: 'Available commands updated',
            },
          ]
        case 'current_mode_update':
          return [
            {
              ...baseEvent(sessionId, provider, 'mode'),
              currentMode: update as CurrentModeUpdate,
              title: 'Agent mode updated',
            },
          ]
        case 'config_option_update':
          return [
            {
              ...baseEvent(sessionId, provider, 'config'),
              configUpdate: update as ConfigOptionUpdate,
              title: 'Configuration options updated',
            },
          ]
        case 'session_info_update':
          return [
            {
              ...baseEvent(sessionId, provider, 'session.info'),
              sessionInfo: update as SessionInfoUpdate,
              title: 'Session info updated',
            },
          ]
        case 'usage_update':
          return [
            {
              ...baseEvent(sessionId, provider, 'usage'),
              usage: update as UsageUpdate,
              title: 'Usage updated',
            },
          ]
        default:
          return [
            {
              ...baseEvent(sessionId, provider, 'debug'),
              title: 'Unhandled session update',
              raw: update,
            },
          ]
      }
    }
    case 'permission_request':
      return [
        {
          ...baseEvent(sessionId, provider, 'permission.requested'),
          title: event.request.toolCall.title ?? undefined,
          toolCallId: event.request.toolCall.toolCallId,
          permissionOptions: event.request.options.map((option) => ({
            optionId: option.optionId,
            name: option.name,
            kind: option.kind,
          })),
          status: 'requested',
          raw: event.request,
        },
      ]
    case 'permission_resolved':
      return [
        {
          ...baseEvent(sessionId, provider, 'permission.resolved'),
          title: event.request.toolCall.title ?? undefined,
          toolCallId: event.request.toolCall.toolCallId,
          permissionSelection: event.selectedOptionId,
          status: 'resolved',
          raw: event.request,
        },
      ]
    case 'prompt_started':
      return [
        {
          ...baseEvent(sessionId, provider, 'status'),
          title: 'Prompt started',
          text: event.prompt,
          status: 'running',
        },
      ]
    case 'prompt_response':
      return [
        {
          ...baseEvent(sessionId, provider, 'status'),
          title: 'Prompt completed',
          promptStopReason: event.response.stopReason,
          status: event.response.stopReason === 'cancelled' ? 'cancelled' : 'completed',
          raw: event.response,
        },
      ]
    case 'stderr':
      return [
        {
          ...baseEvent(sessionId, provider, 'debug'),
          title: 'codex-acp stderr',
          text: event.text,
        },
      ]
    case 'process_exit':
      return [
        {
          ...baseEvent(sessionId, provider, 'status'),
          title: 'Provider process exited',
          text: `code=${String(event.code)} signal=${String(event.signal)}`,
          status: event.code === 0 ? 'completed' : 'failed',
          raw: event,
        },
      ]
    case 'error':
      return [
        {
          ...baseEvent(sessionId, provider, 'error'),
          title: 'Provider error',
          text: event.message,
          status: 'failed',
          raw: event.raw,
        },
      ]
    default:
      return [
        {
          ...baseEvent(sessionId, provider, 'debug'),
          title: 'Unknown provider event',
          raw: event,
        },
      ]
  }
}
