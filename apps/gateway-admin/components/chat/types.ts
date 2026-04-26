import type { BridgeEvent, BridgeSessionStatus } from '@/lib/acp/types'
import type { AttachmentRef } from '@/lib/fs/types'

export type { AttachmentRef }

export interface ACPAgent {
  id: string
  name: string
  description: string
  version: string
  capabilities: string[]
}

export interface ACPProject {
  id: string
  name: string
  agentId: string
  collapsed?: boolean
}

export type ACPRunStatus = BridgeSessionStatus

export interface ACPRun {
  id: string
  projectId: string
  agentId: string
  provider: string
  title: string
  createdAt: Date
  updatedAt: Date
  status: ACPRunStatus
  providerSessionId: string
  cwd: string
}

export interface TranscriptToolCall {
  id: string
  title: string
  status?: 'pending' | 'in_progress' | 'completed' | 'failed' | 'idle' | 'running' | 'cancelled'
  kind?: string | null
  input?: unknown
  output?: unknown
  content?: unknown[] | null
  locations: string[]
}

export interface ACPMessage {
  id: string
  runId: string
  role: 'user' | 'assistant' | 'system'
  text: string
  createdAt: Date
  isStreaming?: boolean
  thoughts: string[]
  toolCalls: TranscriptToolCall[]
}

export type ActivityItem = BridgeEvent
