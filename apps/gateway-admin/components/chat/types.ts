// ACP (Agent Client Protocol) shaped types for the chat UI mockup.
// Shaped to match ACP's run/message/part model so wiring up a real backend is minimal.

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

export type ACPRunStatus = 'running' | 'completed' | 'failed' | 'cancelled'

export interface ACPRun {
  id: string
  projectId: string
  agentId: string
  title: string
  createdAt: Date
  updatedAt: Date
  status: ACPRunStatus
}

export type ACPPartType = 'text' | 'tool_use' | 'tool_result' | 'thinking'

export interface ACPTextPart {
  type: 'text'
  text: string
}

export interface ACPThinkingPart {
  type: 'thinking'
  thinking: string
}

export interface ACPToolUsePart {
  type: 'tool_use'
  id: string
  name: string
  input: Record<string, unknown>
}

export interface ACPToolResultPart {
  type: 'tool_result'
  tool_use_id: string
  name: string
  content: string
  is_error?: boolean
}

export type ACPPart = ACPTextPart | ACPThinkingPart | ACPToolUsePart | ACPToolResultPart

export type ACPRole = 'user' | 'assistant'

export interface ACPMessage {
  id: string
  runId: string
  role: ACPRole
  parts: ACPPart[]
  createdAt: Date
  isStreaming?: boolean
}

export interface ChatState {
  projects: ACPProject[]
  runs: ACPRun[]
  messages: Record<string, ACPMessage[]>
  agents: ACPAgent[]
  selectedRunId: string | null
  selectedProjectId: string | null
}
