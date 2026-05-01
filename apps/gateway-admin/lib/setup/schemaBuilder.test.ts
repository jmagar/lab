import test from 'node:test'
import assert from 'node:assert/strict'

import {
  DEFAULT_UI,
  buildSchema,
  schemaVersion,
  stripBlankSecrets,
  type FieldView,
} from './schemaBuilder.ts'

function field(overrides: Partial<FieldView> = {}): FieldView {
  return {
    name: 'FOO',
    description: 'Foo',
    example: '',
    required: false,
    secret: false,
    ui: DEFAULT_UI,
    ...overrides,
  }
}

test('buildSchema flags missing required text fields', () => {
  const schema = buildSchema([field({ name: 'FOO_URL', required: true })])
  const result = schema.safeParse({ FOO_URL: '' })
  assert.equal(result.success, false)
})

test('buildSchema accepts a non-blank required string', () => {
  const schema = buildSchema([field({ name: 'FOO', required: true })])
  const result = schema.safeParse({ FOO: 'bar' })
  assert.equal(result.success, true)
})

test('buildSchema validates url scheme via the url rule', () => {
  const schema = buildSchema([
    field({ name: 'FOO_URL', required: true, ui: { ...DEFAULT_UI, kind: 'url' } }),
  ])
  assert.equal(schema.safeParse({ FOO_URL: 'not a url' }).success, false)
  assert.equal(schema.safeParse({ FOO_URL: 'https://example.com' }).success, true)
})

test('buildSchema accepts blank for secret with stored value', () => {
  const schema = buildSchema([
    field({
      name: 'FOO_API_KEY',
      required: true,
      secret: true,
      hasStoredSecret: true,
      ui: { ...DEFAULT_UI, kind: 'secret' },
    }),
  ])
  // Blank means "keep stored value" — must succeed.
  assert.equal(schema.safeParse({ FOO_API_KEY: '' }).success, true)
})

test('buildSchema rejects path traversal on file_path', () => {
  const schema = buildSchema([
    field({ name: 'DATA_DIR', ui: { ...DEFAULT_UI, kind: 'file_path' } }),
  ])
  assert.equal(schema.safeParse({ DATA_DIR: '../etc' }).success, false)
  assert.equal(schema.safeParse({ DATA_DIR: 'data/lab' }).success, true)
})

test('buildSchema enforces enum allowlist', () => {
  const schema = buildSchema([
    field({
      name: 'MODE',
      ui: { ...DEFAULT_UI, kind: 'enum', enum_values: ['plugin', 'native'] },
    }),
  ])
  assert.equal(schema.safeParse({ MODE: 'plugin' }).success, true)
  assert.equal(schema.safeParse({ MODE: 'wat' }).success, false)
})

test('schemaVersion is stable when shape unchanged', () => {
  const fields: FieldView[] = [field({ name: 'A' }), field({ name: 'B', secret: true })]
  const v1 = schemaVersion(fields)
  const v2 = schemaVersion([{ ...fields[0]! }, { ...fields[1]! }])
  assert.equal(v1, v2)
})

test('schemaVersion changes when validation tightens', () => {
  const before = schemaVersion([field({ name: 'A' })])
  const after = schemaVersion([
    field({
      name: 'A',
      ui: { ...DEFAULT_UI, validation: { ...DEFAULT_UI.validation, required: true } },
    }),
  ])
  assert.notEqual(before, after)
})

test('stripBlankSecrets keeps stored secrets when blank submitted', () => {
  const fields = [
    field({ name: 'FOO_API_KEY', secret: true, hasStoredSecret: true }),
    field({ name: 'FOO_URL' }),
  ]
  const out = stripBlankSecrets({ FOO_API_KEY: '', FOO_URL: 'https://x' }, fields)
  assert.deepEqual(out, { FOO_URL: 'https://x' })
})

test('stripBlankSecrets keeps non-blank secrets', () => {
  const fields = [field({ name: 'FOO_API_KEY', secret: true, hasStoredSecret: true })]
  const out = stripBlankSecrets({ FOO_API_KEY: 'real-secret' }, fields)
  assert.deepEqual(out, { FOO_API_KEY: 'real-secret' })
})
