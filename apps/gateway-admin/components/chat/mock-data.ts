import type { ACPAgent, ACPMessage, ACPProject, ACPRun, ChatState } from './types'

const now = new Date()
const minsAgo = (n: number) => new Date(now.getTime() - n * 60_000)
const hoursAgo = (n: number) => new Date(now.getTime() - n * 3_600_000)
const daysAgo = (n: number) => new Date(now.getTime() - n * 86_400_000)

export const MOCK_AGENTS: ACPAgent[] = [
  {
    id: 'claude-3-7',
    name: 'Claude 3.7 Sonnet',
    description: 'Anthropic Claude via lab gateway',
    version: '3.7',
    capabilities: ['tool_use', 'vision', 'streaming'],
  },
  {
    id: 'gpt-4o',
    name: 'GPT-4o',
    description: 'OpenAI GPT-4o via lab gateway',
    version: '4o',
    capabilities: ['tool_use', 'vision', 'streaming'],
  },
  {
    id: 'gemini-2-flash',
    name: 'Gemini 2.0 Flash',
    description: 'Google Gemini via lab gateway',
    version: '2.0-flash',
    capabilities: ['tool_use', 'vision', 'streaming'],
  },
]

export const MOCK_PROJECTS: ACPProject[] = [
  { id: 'lab', name: 'lab', agentId: 'claude-3-7' },
  { id: 'swag', name: 'swag', agentId: 'claude-3-7' },
  { id: 'pr-fixer', name: 'pr-fixer', agentId: 'gpt-4o', collapsed: false },
  { id: 'monitor', name: 'monitor', agentId: 'claude-3-7' },
  { id: 'agent-stash', name: 'agent-stash', agentId: 'gemini-2-flash' },
  { id: 'noxa', name: 'noxa', agentId: 'claude-3-7' },
]

export const MOCK_RUNS: ACPRun[] = [
  // lab project
  {
    id: 'run-lab-1',
    projectId: 'lab',
    agentId: 'claude-3-7',
    title: '/lavra-brainstorm I want to work on creat…',
    createdAt: minsAgo(8),
    updatedAt: minsAgo(8),
    status: 'running',
  },
  {
    id: 'run-lab-2',
    projectId: 'lab',
    agentId: 'claude-3-7',
    title: '/lavra-eng-review the plan and address Al…',
    createdAt: minsAgo(20),
    updatedAt: minsAgo(20),
    status: 'completed',
  },
  {
    id: 'run-lab-3',
    projectId: 'lab',
    agentId: 'claude-3-7',
    title: 'hi',
    createdAt: minsAgo(22),
    updatedAt: minsAgo(22),
    status: 'completed',
  },
  // swag project
  {
    id: 'run-swag-1',
    projectId: 'swag',
    agentId: 'claude-3-7',
    title: 'review and update accordingly - is…',
    createdAt: minsAgo(9),
    updatedAt: minsAgo(9),
    status: 'completed',
  },
  // pr-fixer project (collapsed, no sessions shown)
  {
    id: 'run-prfixer-1',
    projectId: 'pr-fixer',
    agentId: 'gpt-4o',
    title: 'Fix type errors in dispatch layer',
    createdAt: hoursAgo(2),
    updatedAt: hoursAgo(2),
    status: 'completed',
  },
  // monitor project
  {
    id: 'run-monitor-1',
    projectId: 'monitor',
    agentId: 'claude-3-7',
    title: '/agents',
    createdAt: hoursAgo(5),
    updatedAt: hoursAgo(5),
    status: 'completed',
  },
  // agent-stash project
  {
    id: 'run-agentstash-1',
    projectId: 'agent-stash',
    agentId: 'gemini-2-flash',
    title: 'Conduct a deep and all encompassing po…',
    createdAt: daysAgo(1),
    updatedAt: daysAgo(1),
    status: 'completed',
  },
  // noxa project
  {
    id: 'run-noxa-1',
    projectId: 'noxa',
    agentId: 'claude-3-7',
    title: '/full-review',
    createdAt: minsAgo(56),
    updatedAt: minsAgo(56),
    status: 'completed',
  },
]

export const MOCK_MESSAGES: Record<string, ACPMessage[]> = {
  'run-lab-1': [
    {
      id: 'msg-1',
      runId: 'run-lab-1',
      role: 'user',
      parts: [
        {
          type: 'text',
          text: 'I want to work on creating a set of reusable shadcnUI components — starting with fully interactive mockups.\n\nThey\'re going to be UI components for an assistant based chat ui',
        },
      ],
      createdAt: minsAgo(8),
    },
    {
      id: 'msg-2',
      runId: 'run-lab-1',
      role: 'agent',
      parts: [
        {
          type: 'thinking',
          thinking: 'The user wants to build a chat UI component library using shadcnUI. I should ask about the target integration — is this for gateway-admin? What specific components do they have in mind? Let me start with a broad vision question.',
        },
        {
          type: 'text',
          text: 'Let me do a quick scan of the workspace to understand the existing context before diving in.',
        },
        {
          type: 'tool_use',
          id: 'tool-1',
          name: 'bash',
          input: { command: 'ls /home/jmagar/workspace/lab/apps/gateway-admin/app' },
        },
      ],
      createdAt: minsAgo(7),
    },
    {
      id: 'msg-3',
      runId: 'run-lab-1',
      role: 'agent',
      parts: [
        {
          type: 'tool_result',
          tool_use_id: 'tool-1',
          name: 'bash',
          content: '(admin)/\nglobals.css\nlayout.tsx\npage.tsx',
        },
        {
          type: 'text',
          text: 'I can see there\'s already a Next.js app (`apps/gateway-admin`) with shadcnUI in this workspace — good context. Requirements here are deliberately vague, so let me explore with you.\n\n**First question:** Where does this chat UI live? I see a few natural fits:\n\na) **New standalone app** — a dedicated chat UI package separate from gateway-admin\nb) **Inside gateway-admin** — add a chat section/page to the existing admin app\nc) **Standalone component library** — a reusable package\nd) **Somewhere else entirely**',
        },
      ],
      createdAt: minsAgo(7),
    },
    {
      id: 'msg-4',
      runId: 'run-lab-1',
      role: 'user',
      parts: [
        {
          type: 'text',
          text: 'well they will be used in gateway-admin... eventually... i just want to play around with getting all the components how i wanted them before i started trying to wire it up too',
        },
      ],
      createdAt: minsAgo(6),
    },
    {
      id: 'msg-5',
      runId: 'run-lab-1',
      role: 'agent',
      parts: [
        {
          type: 'text',
          text: 'Got it — design-first, mockups before plumbing. Smart approach.\n\n**Second question:** What components are you picturing for this chat UI?',
        },
      ],
      createdAt: minsAgo(6),
      isStreaming: true,
    },
  ],
  'run-lab-2': [
    {
      id: 'msg-a1',
      runId: 'run-lab-2',
      role: 'user',
      parts: [{ type: 'text', text: '/lavra-eng-review the plan and address all comments' }],
      createdAt: minsAgo(20),
    },
    {
      id: 'msg-a2',
      runId: 'run-lab-2',
      role: 'agent',
      parts: [
        {
          type: 'thinking',
          thinking: 'The user wants a comprehensive engineering review of the current plan. I should load the eng-review skill and systematically go through the plan, addressing all open comments.',
        },
        {
          type: 'tool_use',
          id: 'tool-a1',
          name: 'read_file',
          input: { path: '.omc/plans/current-plan.md' },
        },
        {
          type: 'tool_result',
          tool_use_id: 'tool-a1',
          name: 'read_file',
          content: '# Plan: Dispatch Layer Refactor\n\n## Phase 1: Shared dispatch module\n...',
        },
        {
          type: 'text',
          text: 'I\'ve reviewed the plan. Here are my engineering findings:\n\n**Critical issues:**\n1. The `InstancePool<C>` generic bound is too broad — it accepts any `Clone` type, but the pool should constrain to `ServiceClient` implementors\n2. Circuit breaker state is not persisted across restarts — consider storing to `~/.lab/.env` or using an in-memory bounded deque\n\n**Non-blocking improvements:**\n- The `dispatch.rs` entry point could export a `prelude` module to simplify imports in CLI/MCP shims\n- Error envelope `kind` strings are duplicated between `mcp/error.rs` and `api/error.rs` — extract to a shared constants module',
        },
      ],
      createdAt: minsAgo(18),
    },
  ],
  'run-swag-1': [
    {
      id: 'msg-b1',
      runId: 'run-swag-1',
      role: 'user',
      parts: [{ type: 'text', text: 'review and update accordingly - is the aurora contract fully satisfied?' }],
      createdAt: minsAgo(9),
    },
    {
      id: 'msg-b2',
      runId: 'run-swag-1',
      role: 'agent',
      parts: [
        {
          type: 'tool_use',
          id: 'tool-b1',
          name: 'glob',
          input: { pattern: 'apps/gateway-admin/components/**/*.tsx' },
        },
        {
          type: 'tool_result',
          tool_use_id: 'tool-b1',
          name: 'glob',
          content: '42 files matched',
        },
        {
          type: 'text',
          text: 'After reviewing all components against the Aurora contract, I found **3 violations**:\n\n1. `components/gateway/status-badge.tsx:12` — uses hardcoded `#00e676` instead of `var(--aurora-success)`\n2. `components/registry/registry-table.tsx:34` — font-family not specified; will fall back to system-ui instead of Inter\n3. `components/setup/setup-wizard.tsx:89` — panel uses `rounded-lg` (8px) instead of `rounded-aurora-2` (18px)\n\nUpdating all three now.',
        },
      ],
      createdAt: minsAgo(8),
    },
  ],
  'run-monitor-1': [
    {
      id: 'msg-c1',
      runId: 'run-monitor-1',
      role: 'user',
      parts: [{ type: 'text', text: '/agents' }],
      createdAt: hoursAgo(5),
    },
    {
      id: 'msg-c2',
      runId: 'run-monitor-1',
      role: 'agent',
      parts: [
        {
          type: 'text',
          text: 'Here are the currently registered agents in your lab gateway:\n\n| Agent | Status | Latency | Requests/hr |\n|-------|--------|---------|-------------|\n| claude-3-7-sonnet | ✓ healthy | 342ms | 847 |\n| gpt-4o | ✓ healthy | 891ms | 203 |\n| gemini-2-flash | ✓ healthy | 189ms | 124 |\n| mistral-large | ⚠ degraded | 4.2s | 12 |\n\n**mistral-large** is experiencing elevated latency. The upstream endpoint has been returning 429s intermittently since 03:14 UTC.',
        },
      ],
      createdAt: hoursAgo(5),
    },
  ],
  'run-noxa-1': [
    {
      id: 'msg-d1',
      runId: 'run-noxa-1',
      role: 'user',
      parts: [{ type: 'text', text: '/full-review' }],
      createdAt: minsAgo(56),
    },
    {
      id: 'msg-d2',
      runId: 'run-noxa-1',
      role: 'agent',
      parts: [
        {
          type: 'thinking',
          thinking: 'Running the full-review skill. This is comprehensive — architect, code, security, and test coverage reviewers all in parallel.',
        },
        {
          type: 'tool_use',
          id: 'tool-d1',
          name: 'dispatch_agents',
          input: {
            agents: ['architect-review', 'code-reviewer', 'security-auditor', 'pr-test-analyzer'],
            mode: 'parallel',
          },
        },
        {
          type: 'tool_result',
          tool_use_id: 'tool-d1',
          name: 'dispatch_agents',
          content: 'All 4 agents dispatched. Awaiting results...',
        },
        {
          type: 'text',
          text: '# Full Review — noxa\n\n## Architecture (architect-review)\n\nOverall architecture is sound. The scrape/crawl/extract separation is clean. One concern: the `research` tool conflates search and synthesis — consider splitting into `search` (pure retrieval) and `synthesize` (generates from gathered sources).\n\n## Code Quality (code-reviewer)\n\n**7 findings, 2 critical:**\n1. `tools/crawl.ts:88` — unchecked `process.env.FIRECRAWL_KEY` access; will throw at runtime if key absent\n2. `tools/research.ts:142` — `await Promise.all` with no error boundary; one failing fetch silently drops the result\n\n## Security (security-auditor)\n\nNo critical vulnerabilities. Medium: the `/batch` endpoint accepts arbitrary URLs with no allowlist — consider domain restrictions.\n\n## Tests (pr-test-analyzer)\n\nTest coverage is 43%. The `extract` and `summarize` tools have zero test coverage.',
        },
      ],
      createdAt: minsAgo(50),
    },
  ],
}

export function createInitialChatState(): ChatState {
  return {
    projects: MOCK_PROJECTS,
    runs: MOCK_RUNS,
    messages: MOCK_MESSAGES,
    agents: MOCK_AGENTS,
    selectedRunId: 'run-lab-1',
    selectedProjectId: 'lab',
  }
}

export function formatTimeAgo(date: Date): string {
  const diffMs = Date.now() - date.getTime()
  const diffMins = Math.floor(diffMs / 60_000)
  const diffHours = Math.floor(diffMs / 3_600_000)
  const diffDays = Math.floor(diffMs / 86_400_000)

  if (diffMins < 60) return `${diffMins}m`
  if (diffHours < 24) return `${diffHours}h`
  return `${diffDays}d`
}
