import test from 'node:test'
import assert from 'node:assert/strict'

import { getDisplayText, getInlineArtifact, getToolPresentation } from './tool-call-presentation'
import type { TranscriptToolCall } from './types'

function toolCall(overrides: Partial<TranscriptToolCall> = {}): TranscriptToolCall {
  return {
    id: 'tool-1',
    title: 'Inspect workspace',
    status: 'completed',
    kind: 'search',
    input: undefined,
    output: undefined,
    content: null,
    locations: [],
    permissionOptions: [],
    permissionSelection: null,
    ...overrides,
  }
}

test('getInlineArtifact derives terminal, file tree, preview, and code-block artifacts', () => {
  const artifact = getInlineArtifact(
    toolCall({
      title: 'Run command',
      kind: 'command',
      input: {
        parsed_cmd: [
          { cmd: 'cat Cargo.toml', path: '/repo/Cargo.toml', type: 'read' },
          { cmd: 'cat src/main.rs', path: '/repo/src/main.rs', type: 'read' },
        ],
      },
      output: '$ cargo check\nFinished dev profile in 0.80s\nhttp://127.0.0.1:3000/preview',
      locations: ['/repo/Cargo.toml', '/repo/src/main.rs'],
    }),
  )

  assert.equal(artifact.terminalOutput?.includes('cargo check'), true)
  assert.equal(artifact.webPreviewUrl, 'http://127.0.0.1:3000/preview')
  assert.deepEqual(artifact.fileTreePaths, ['/repo/Cargo.toml', '/repo/src/main.rs'])
  assert.equal(artifact.codeBlock?.path, '/repo/Cargo.toml')
})

test('getInlineArtifact preserves multiline structured fields for code and terminal artifacts', () => {
  const artifact = getInlineArtifact(
    toolCall({
      title: 'Apply patch',
      kind: 'edit',
      input: {
        patch: '*** Begin Patch\n*** Update File: src/app.ts\n+const ready = true\n*** End Patch',
      },
      output: {
        stdout: '$ pnpm lint\nAll checks passed',
      },
      locations: ['/repo/src/app.ts'],
    }),
  )

  assert.equal(artifact.codeBlock?.path, '/repo/src/app.ts')
  assert.match(artifact.codeBlock?.code ?? '', /\*\*\* Begin Patch/)
  assert.equal(artifact.codeBlock?.language, 'ts')
  assert.match(artifact.terminalOutput ?? '', /\$ pnpm lint/)
})

test('getToolPresentation treats permission-kind tool calls as permission timeline items', () => {
  const call = toolCall({
    title: 'Read ~/.ssh/config',
    kind: 'permission',
    permissionOptions: [{ optionId: 'allow_once', name: 'Allow once', kind: 'allow_once' }],
    permissionSelection: 'allow_once',
  })

  const presentation = getToolPresentation(call, getInlineArtifact(call))

  assert.equal(presentation.category, 'permission')
  assert.equal(presentation.label, 'Read ~/.ssh/config')
})

// ---------------------------------------------------------------------------
// Security tests for terminal output sanitization (S9 / R1 / R7 / O3)
// ---------------------------------------------------------------------------

test('getInlineArtifact prefers streamed terminal output over raw output string (A8)', () => {
  // A8: toolCall.terminal != null check (NOT truthy) should prefer terminal.
  // Even with empty rawChunks, presence of terminal field triggers the ACP path.
  const call = toolCall({
    terminal: {
      rawChunks: ['$ cargo build\n', 'Compiling lab v0.11.1\n'],
      totalBytes: 40,
      truncated: false,
      exitCode: null,
    },
    output: 'this should NOT be used as terminal output',
  })

  const artifact = getInlineArtifact(call)
  assert.ok(artifact.terminalOutput?.includes('cargo build'), 'should use ACP terminal rawChunks')
  assert.ok(!artifact.terminalOutput?.includes('NOT be used'), 'should NOT use raw output string')
})

test('getInlineArtifact applies TERMINAL_RENDER_TAIL_BYTES at concatenation (O3)', () => {
  // Create rawChunks that exceed 256 KiB total.
  const bigChunk = 'x'.repeat(200 * 1024) // 200 KB
  const call = toolCall({
    terminal: {
      rawChunks: [bigChunk, bigChunk], // 400 KB total
      totalBytes: 400 * 1024,
      truncated: true,
      exitCode: 0,
    },
  })

  const artifact = getInlineArtifact(call)
  const outputLen = artifact.terminalOutput?.length ?? 0
  assert.ok(
    outputLen <= 262_144,
    `terminalOutput length ${outputLen} should be <= TERMINAL_RENDER_TAIL_BYTES (262144)`,
  )
})

test('getDisplayText strips OSC sequences (R7)', () => {
  // OSC-8 hyperlink with javascript: URI (R7 hostile input)
  const withOscJavascript = '\x1b]8;;javascript:alert(1)\x07click me\x1b]8;;\x07'
  const withOscData = '\x1b]8;;data:text/html,<h1>XSS</h1>\x1b\\click me\x1b]8;;\x1b\\'
  // OSC-52 clipboard escape
  const withOsc52 = '\x1b]52;c;c2Vuc2l0aXZlCg==\x07'

  const result1 = getDisplayText([withOscJavascript])
  const result2 = getDisplayText([withOscData])
  const result3 = getDisplayText([withOsc52])

  assert.ok(!result1.includes('javascript:'), 'javascript: URI must be stripped')
  assert.ok(!result2.includes('data:text/html'), 'data: URI must be stripped')
  assert.ok(!result3.includes('c2Vuc2l0aXZlCg'), 'OSC-52 clipboard data must be stripped')
})

test('getDisplayText strips ANSI escape sequences (R1)', () => {
  // ANSI color codes and control sequences
  const withAnsi = '\x1b[31mred text\x1b[0m normal \x1b[1;32mbold green\x1b[0m'

  const result = getDisplayText([withAnsi])
  assert.ok(!result.includes('\x1b['), 'ANSI escape sequences must be stripped')
  assert.ok(result.includes('red text'), 'plain text content must be preserved')
  assert.ok(result.includes('normal'), 'plain text content must be preserved')
  assert.ok(result.includes('bold green'), 'plain text content must be preserved')
})
