import test from 'node:test'
import assert from 'node:assert/strict'

import { getInlineArtifact, getToolPresentation } from './tool-call-presentation'
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
