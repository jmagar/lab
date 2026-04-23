import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { TextSurface } from './text-surface'
import type { EditorDiagnostic } from '@/lib/editor/types'

const diagnostics: EditorDiagnostic[] = [
  {
    from: 0,
    to: 3,
    severity: 'warning',
    message: 'Example warning',
  },
]

test('renders read-only documents with line numbers and copy action', () => {
  const markup = renderToStaticMarkup(
    <TextSurface
      path="plugin.json"
      value={'{"name":"demo"}'}
      mode="view"
      language="json"
      onCopy={() => {}}
    />,
  )

  assert.match(markup, /plugin.json/)
  assert.match(markup, /Copy/)
  assert.match(markup, /aurora-text-surface/)
  assert.match(markup, /aria-label="Code editor"/)
})

test('renders editable documents and reports dirty state in toolbar', () => {
  const markup = renderToStaticMarkup(
    <TextSurface
      path="README.md"
      value={'# Demo'}
      mode="edit"
      language="markdown"
      dirty
      onChange={() => {}}
      onSave={() => {}}
    />,
  )

  assert.match(markup, /Dirty/)
  assert.match(markup, /Save/)
  assert.match(markup, /README.md/)
})

test('shows validation status in the toolbar', () => {
  const markup = renderToStaticMarkup(
    <TextSurface
      path="agent.md"
      value={'---\nname: demo\n---'}
      mode="edit"
      language="markdown"
      diagnostics={diagnostics}
    />,
  )

  assert.match(markup, /Example warning/)
  assert.match(markup, /Warning/i)
})

test('applies Aurora editor classes instead of prism markup', () => {
  const markup = renderToStaticMarkup(
    <TextSurface
      path="config.toml"
      value={'title = "demo"'}
      mode="view"
      language="toml"
    />,
  )

  assert.match(markup, /cm-editor/)
  assert.match(markup, /aurora-text-surface/)
  assert.doesNotMatch(markup, /token punctuation/)
})
