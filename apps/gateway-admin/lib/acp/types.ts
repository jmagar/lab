import type {
  AgentCapabilities,
  AvailableCommand,
  ClientCapabilities,
  ConfigOptionUpdate,
  CurrentModeUpdate,
  PlanEntry,
  PromptResponse,
  RequestPermissionRequest,
  SessionInfoUpdate,
  SessionMode,
  SessionNotification,
  ToolCallContent,
  ToolCallStatus,
  ToolKind,
  UsageUpdate,
} from '@agentclientprotocol/sdk'

export type AcpProviderKind = string
export type BridgeSessionStatus = 'idle' | 'running' | 'completed' | 'failed' | 'cancelled'

export type ProviderHealth = {
  provider: AcpProviderKind
  ready: boolean
  command: string
  args: string[]
  message: string
}

export type BridgeSessionSummary = {
  id: string
  providerSessionId: string
  provider: AcpProviderKind
  title: string
  cwd: string
  createdAt: string
  updatedAt: string
  status: BridgeSessionStatus
  agentName: string
  agentVersion: string
  resumable?: boolean
}

export type BridgePermissionOption = {
  optionId: string
  name: string
  kind: string
}

export type BridgeEventKind =
  | 'message.chunk'
  | 'tool.call'
  | 'tool.update'
  | 'plan'
  | 'permission.requested'
  | 'permission.resolved'
  | 'session.info'
  | 'usage'
  | 'commands'
  | 'mode'
  | 'config'
  | 'status'
  | 'error'
  | 'debug'

export type BridgeEvent = {
  id: string
  seq: number
  sessionId: string
  provider: AcpProviderKind
  kind: BridgeEventKind
  createdAt: string
  role?: 'user' | 'assistant' | 'thinking'
  messageId?: string | null
  text?: string
  title?: string
  status?: BridgeSessionStatus | ToolCallStatus | 'requested' | 'resolved'
  toolCallId?: string
  toolKind?: ToolKind | null
  rawInput?: unknown
  rawOutput?: unknown
  toolContent?: ToolCallContent[] | null
  locations?: string[]
  plan?: PlanEntry[]
  permissionOptions?: BridgePermissionOption[]
  permissionSelection?: string | null
  sessionInfo?: SessionInfoUpdate
  usage?: UsageUpdate
  commands?: AvailableCommand[]
  currentMode?: CurrentModeUpdate
  configUpdate?: ConfigOptionUpdate
  promptStopReason?: PromptResponse['stopReason']
  raw?: unknown
}

export type ProviderRuntimeEvent =
  | { type: 'session_notification'; notification: SessionNotification }
  | { type: 'permission_request'; request: RequestPermissionRequest }
  | {
      type: 'permission_resolved'
      request: RequestPermissionRequest
      selectedOptionId: string | null
    }
  | { type: 'prompt_started'; prompt: string }
  | { type: 'prompt_response'; response: PromptResponse }
  | { type: 'stderr'; text: string }
  | { type: 'error'; message: string; raw?: unknown }
  | { type: 'process_exit'; code: number | null; signal: string | null }

export type StartSessionInput = {
  cwd: string
  title?: string
  clientCapabilities?: ClientCapabilities
}

export type StartSessionResult = {
  providerSessionId: string
  agentName: string
  agentVersion: string
  capabilities?: AgentCapabilities
}
