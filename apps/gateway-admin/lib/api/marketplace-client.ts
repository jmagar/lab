import type { Marketplace, Plugin, Artifact, ArtifactLang } from '../types/marketplace.js'

const MOCK_MARKETPLACES: Marketplace[] = [
  {
    id: 'claude-plugins-official',
    name: 'Claude Plugins Official',
    owner: 'Anthropic',
    ghUser: 'anthropics',
    repo: 'anthropics/claude-plugins-official',
    source: 'github',
    desc: 'Official Anthropic extensions — LSPs, code review, AI productivity plugins, MCP integrations, and more.',
    autoUpdate: false,
    totalPlugins: 48,
    lastUpdated: '2026-04-20T05:07:20.607Z',
  },
  {
    id: 'superpowers-marketplace',
    name: 'Superpowers',
    owner: 'Jesse Vincent',
    ghUser: 'obra',
    repo: 'obra/superpowers-marketplace',
    source: 'github',
    desc: 'Skills, workflows, and productivity tools. TDD, debugging, collaboration patterns, and proven development techniques.',
    autoUpdate: true,
    totalPlugins: 8,
    lastUpdated: '2026-04-21T19:13:23.871Z',
  },
  {
    id: 'claude-code-workflows',
    name: 'Claude Code Workflows',
    owner: 'Seth Hobson',
    ghUser: 'wshobson',
    repo: 'wshobson/agents',
    source: 'github',
    desc: '79 focused plugins, 184 specialized agents, and 150 skills. Optimized for granular installation.',
    autoUpdate: true,
    totalPlugins: 79,
    lastUpdated: '2026-04-21T19:26:20.085Z',
  },
  {
    id: 'jmagar-lab',
    name: 'jmagar Lab',
    owner: 'Jacob Magar',
    ghUser: 'jmagar',
    source: 'local',
    path: '~/.claude-plugin/marketplace.json',
    desc: 'Homelab control plane — CLI, MCP server, HTTP API for 24 services.',
    autoUpdate: false,
    totalPlugins: 2,
    lastUpdated: '2026-04-22T01:06:42.986Z',
  },
]

const MOCK_PLUGINS: Plugin[] = [
  { id: 'code-review@cpo', name: 'code-review', mkt: 'claude-plugins-official', ver: '1.0.0', desc: 'Elite code review expert — AI-powered analysis, security vulnerabilities, performance optimization.', tags: ['review', 'security', 'quality'], installed: true, installedAt: '2026-01-14', updatedAt: '2026-04-20' },
  { id: 'typescript-lsp@cpo', name: 'typescript-lsp', mkt: 'claude-plugins-official', ver: '1.0.0', desc: 'TypeScript Language Server — type checking, go-to-definition, hover types, and diagnostics inside Claude Code.', tags: ['lsp', 'typescript', 'ide'], installed: true, installedAt: '2026-01-14', updatedAt: '2026-04-01' },
  { id: 'chrome-devtools-mcp@cpo', name: 'chrome-devtools-mcp', mkt: 'claude-plugins-official', ver: 'latest', desc: 'Chrome DevTools MCP — browser automation, DOM inspection, network monitoring.', tags: ['browser', 'devtools', 'mcp'], installed: true, hasUpdate: true, installedAt: '2026-03-14', updatedAt: '2026-04-08' },
  { id: 'git-ops@cpo', name: 'git-ops', mkt: 'claude-plugins-official', ver: '1.1.0', desc: 'Advanced git operations — interactive rebase assistance, conflict resolution, branch strategy.', tags: ['git', 'ops', 'workflow'], installed: false },
  { id: 'security-scanner@cpo', name: 'security-scanner', mkt: 'claude-plugins-official', ver: '2.0.1', desc: 'Static security analysis — SAST scanning, dependency vulnerability checking, secrets detection.', tags: ['security', 'sast', 'vulnerabilities'], installed: false },
  { id: 'superpowers@spm', name: 'superpowers', mkt: 'superpowers-marketplace', ver: '5.0.7', desc: 'Core skills library: TDD, debugging, collaboration patterns, and proven development techniques.', tags: ['skills', 'tdd', 'debugging', 'core'], installed: true, installedAt: '2026-01-14', updatedAt: '2026-04-01' },
  { id: 'omc@spm', name: 'oh-my-claudecode', mkt: 'superpowers-marketplace', ver: '2.1.0', desc: 'Multi-agent orchestration layer — coordinate specialized agents, tools, and skills for complex work.', tags: ['orchestration', 'agents', 'workflow'], installed: false },
  { id: 'tdd@ccw', name: 'tdd-workflows', mkt: 'claude-code-workflows', ver: '1.3.0', desc: 'Test-driven development — red-green-refactor cycle, coverage analysis, and test architecture patterns.', tags: ['tdd', 'testing', 'workflow'], installed: true, installedAt: '2026-03-02', updatedAt: '2026-04-01' },
  { id: 'comp-review@ccw', name: 'comprehensive-review', mkt: 'claude-code-workflows', ver: '1.3.0', desc: 'Multi-perspective code review with architect, security, and performance specialized agents in parallel.', tags: ['review', 'security', 'perf'], installed: true, installedAt: '2026-03-02', updatedAt: '2026-04-01' },
  { id: 'lab@jl', name: 'lab', mkt: 'jmagar-lab', ver: '0.7.0', desc: 'Homelab control plane — CLI, MCP server, HTTP API for 24 services (Radarr, Sonarr, Plex, UniFi, Unraid, and more).', tags: ['homelab', 'mcp', 'cli', 'rust'], installed: true, installedAt: '2026-04-21', updatedAt: '2026-04-22' },
]

const MOCK_ARTIFACTS: Record<string, Artifact[]> = {
  'code-review@cpo': [
    { path: 'plugin.json', lang: 'json', content: JSON.stringify({
      name: 'code-review',
      version: '1.0.0',
      description: 'Elite code review expert — AI-powered analysis, security vulnerabilities, performance optimization.',
      agents: ['agents/code-reviewer.md', 'agents/architect-review.md'],
      commands: ['commands/code-review.md'],
      skills: ['skills/review-checklist.md'],
    }, null, 2) },
    { path: 'agents/code-reviewer.md', lang: 'markdown', content: `---\nname: code-reviewer\ndescription: Elite code review expert specializing in security vulnerabilities and performance optimization.\nsubagent_type: comprehensive-review:code-reviewer\n---\n\nReview code for quality, security, and production reliability.\n` },
    { path: 'agents/architect-review.md', lang: 'markdown', content: `---\nname: architect-review\ndescription: Master software architect specializing in modern architecture patterns and DDD.\nsubagent_type: comprehensive-review:architect-review\n---\n\nReview system designs and code changes for architectural integrity.\n` },
    { path: 'commands/code-review.md', lang: 'markdown', content: `# Code Review\n\nRun a comprehensive multi-agent code review on recent changes.\n\n## Usage\n\nInvoke before committing or creating a PR.\n\n## Options\n\n- \`--base <ref>\` — compare against specific ref (default: HEAD)\n- \`--severity high\` — only report high severity issues\n` },
    { path: 'skills/review-checklist.md', lang: 'markdown', content: `---\nname: review-checklist\ndescription: Use when performing code review to ensure thorough coverage\n---\n\n## Checklist\n\n- [ ] No hardcoded secrets\n- [ ] Input validation at system boundaries\n- [ ] Auth checks on all protected endpoints\n- [ ] All error paths handled\n` },
    { path: 'README.md', lang: 'markdown', content: `# code-review\n\nElite multi-agent code review for Claude Code.\n\n## Agents\n\n| Agent | Focus |\n|-------|-------|\n| \`code-reviewer\` | Quality, security, performance |\n| \`architect-review\` | SOLID principles, architectural debt |\n\n## Usage\n\n\`\`\`\n/code-review\n/code-review --base main\n\`\`\`\n` },
  ],
  'superpowers@spm': [
    { path: 'plugin.json', lang: 'json', content: JSON.stringify({
      name: 'superpowers',
      version: '5.0.7',
      description: 'Core skills library: TDD, debugging, collaboration patterns.',
      skills: ['skills/tdd.md', 'skills/debugging.md', 'skills/brainstorming.md'],
      commands: ['commands/tdd.md', 'commands/debug.md'],
    }, null, 2) },
    { path: 'skills/tdd.md', lang: 'markdown', content: `---\nname: tdd\ndescription: Use when implementing any new feature or fixing any bug using TDD\n---\n\n# Test-Driven Development\n\n**Red → Green → Refactor**\n\n1. Write the smallest failing test\n2. Write the minimum code to make it pass\n3. Refactor while keeping tests green\n` },
    { path: 'skills/debugging.md', lang: 'markdown', content: `---\nname: debugging\ndescription: Use when stuck on a bug you cannot solve\n---\n\n# Debugging Protocol\n\n## Phase 1: Reproduce\n\nNever fix what you cannot reproduce.\n\n## Phase 2: Isolate\n\nBinary search the problem space.\n\n## Phase 3: Hypothesize and verify\n\nState a hypothesis. Design an experiment to falsify it.\n` },
    { path: 'commands/tdd.md', lang: 'markdown', content: `# TDD\n\nActivate Test-Driven Development mode.\n\n## What Changes\n\n- Every feature starts with a failing test\n- Implementation follows tests, never precedes them\n` },
    { path: 'README.md', lang: 'markdown', content: `# superpowers\n\nCore skills library for Claude Code. Includes TDD, debugging, collaboration patterns, and proven development techniques.\n\n## Skills\n\n- \`tdd\` — Red-green-refactor cycle\n- \`debugging\` — Systematic debugging protocol\n- \`brainstorming\` — Structured ideation\n\n## Installation\n\n\`\`\`\nclaude plugin install superpowers\n\`\`\`\n` },
  ],
}

export function detectArtifactLang(path: string): ArtifactLang {
  if (path.endsWith('.json')) return 'json'
  if (path.endsWith('.yaml') || path.endsWith('.yml')) return 'yaml'
  if (path.endsWith('.md')) return 'markdown'
  if (path.endsWith('.sh') || path.endsWith('.bash') || !path.includes('.')) return 'bash'
  if (path.endsWith('.toml')) return 'toml'
  return 'text'
}

export function getArtifacts(pluginId: string): Artifact[] {
  return MOCK_ARTIFACTS[pluginId] ?? []
}

export async function fetchMarketplaces(): Promise<Marketplace[]> {
  return structuredClone(MOCK_MARKETPLACES)
}

export async function fetchPlugins(): Promise<Plugin[]> {
  return structuredClone(MOCK_PLUGINS)
}

export async function getInstalledPluginIds(): Promise<Set<string>> {
  const plugins = await fetchPlugins()
  return new Set(plugins.filter(p => p.installed).map(p => p.id))
}

export async function installPlugin(pluginId: string): Promise<void> {
  void pluginId
  await new Promise(r => setTimeout(r, 600))
}

export async function uninstallPlugin(pluginId: string): Promise<void> {
  void pluginId
  await new Promise(r => setTimeout(r, 400))
}

export async function addMarketplace(input: {
  repo?: string
  url?: string
  name?: string
  autoUpdate: boolean
}): Promise<Marketplace> {
  await new Promise(r => setTimeout(r, 800))
  const id = input.repo?.replace('/', '-') ?? `custom-${Date.now()}`
  const ghUser = input.repo?.split('/')[0] ?? ''
  return {
    id,
    name: input.name ?? input.repo ?? id,
    owner: ghUser,
    ghUser,
    repo: input.repo,
    url: input.url,
    source: input.repo ? 'github' : 'git',
    desc: 'Custom marketplace',
    autoUpdate: input.autoUpdate,
    totalPlugins: 0,
    lastUpdated: new Date().toISOString(),
  }
}
