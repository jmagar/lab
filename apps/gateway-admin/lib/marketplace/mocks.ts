import type { McpServer, AcpAgent } from './types'

export const MOCK_MCP_SERVERS: McpServer[] = [
  {
    name: 'filesystem',
    description: 'Read and write files on the local filesystem with configurable access controls.',
    version: '0.6.2',
    package: '@modelcontextprotocol/server-filesystem',
    transport: ['stdio'],
  },
  {
    name: 'github',
    description: 'Interact with GitHub repositories, issues, pull requests, and workflows.',
    version: '0.4.1',
    package: '@modelcontextprotocol/server-github',
    transport: ['stdio'],
  },
  {
    name: 'memory',
    description: 'Persistent key-value memory store for long-running agent sessions.',
    version: '0.3.0',
    package: '@modelcontextprotocol/server-memory',
    transport: ['stdio', 'sse'],
  },
  {
    name: 'brave-search',
    description: 'Web and local search via the Brave Search API.',
    version: '0.2.0',
    package: '@modelcontextprotocol/server-brave-search',
    transport: ['stdio'],
  },
]

export const MOCK_ACP_AGENTS: AcpAgent[] = [
  {
    id: 'anthropic/claude-analyst',
    name: 'Claude Analyst',
    version: '1.0.0',
    description: 'Data analysis and visualization agent with code execution support.',
    distribution: { npx: {} },
  },
  {
    id: 'lab/homelab-ops',
    name: 'Homelab Ops',
    version: '0.2.1',
    description: 'Manages Unraid, Docker, and homelab services via lab MCP tools.',
    distribution: { binary: {} },
  },
  {
    id: 'community/research-agent',
    name: 'Research Agent',
    version: '0.5.0',
    description: 'Web research, summarization, and citation extraction.',
    distribution: { uvx: {} },
  },
]
