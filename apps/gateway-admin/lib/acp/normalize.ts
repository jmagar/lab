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

function baseEvent(sessionId: string, provider: AcpProviderKind, kind: BridgeEvent['kind']): Omit<BridgeEvent, 'id' | 'seq'> {
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

function normalizeToolCall(sessionId: string, provider: AcpProviderKind, update: ToolCall): Omit<BridgeEvent, 'id' | 'seq'> {
  return {
    ...baseEvent(sessionId, provider, 'tool.call'),
    title: update.title,
    toolCallId: update.toolCallId,
    toolKind: update.kind ?? null,
    status: update.status ?? 'pending',
    rawInput: update.rawInput,
    rawOutput: update.rawOutput,
    toolContent: update.content ?? null,
    locations: update.locations?.map((location) => location.path) ?? [],
  }
}

function normalizeToolUpdate(sessionId: string, provider: AcpProviderKind, update: ToolCallUpdate): Omit<BridgeEvent, 'id' | 'seq'> {
  return {
    ...baseEvent(sessionId, provider, 'tool.update'),
    title: update.title ?? undefined,
    toolCallId: update.toolCallId,
    toolKind: update.kind ?? null,
    status: update.status ?? undefined,
    rawInput: update.rawInput,
    rawOutput: update.rawOutput,
    toolContent: update.content ?? null,
    locations: update.locations?.map((location) => location.path) ?? [],
  }
}

export function normalizeProviderEvent(
  sessionId: string,
  provider: AcpProviderKind,
  event: ProviderRuntimeEvent,
): Array<Omit<BridgeEvent, 'id' | 'seq'>> {
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
          return [normalizeToolCall(sessionId, provider, update)]
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
