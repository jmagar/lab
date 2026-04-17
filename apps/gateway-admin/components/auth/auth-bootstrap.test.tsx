import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { AuthBootstrap } from './auth-bootstrap.tsx'
import { __setBrowserSessionStateForTests } from '../../lib/auth/session-store.ts'

function withAuthEnv<T>(run: () => T): T {
  const keys = [
    'NEXT_PUBLIC_API_TOKEN',
    'NEXT_PUBLIC_MOCK_DATA',
    'NEXT_PUBLIC_STANDALONE_BEARER_AUTH',
  ] as const
  const previous = new Map(keys.map((key) => [key, process.env[key]]))

  for (const key of keys) {
    delete process.env[key]
  }

  try {
    return run()
  } finally {
    for (const key of keys) {
      const value = previous.get(key)
      if (value === undefined) {
        delete process.env[key]
      } else {
        process.env[key] = value
      }
    }
  }
}

test('AuthBootstrap renders the auth_error login screen instead of the generic error state', () => {
  __setBrowserSessionStateForTests({
    status: 'auth_error',
    kind: 'internal_error',
    message: 'auth store unavailable',
    requestId: 'req-auth-123',
  })

  const markup = withAuthEnv(() =>
    renderToStaticMarkup(
      React.createElement(
        AuthBootstrap,
        null,
        React.createElement('div', null, 'children'),
      ),
    ),
  )

  assert.match(markup, /Authentication Error/)
  assert.match(markup, /auth store unavailable/)
  assert.match(markup, /Request ID: req-auth-123/)
  assert.match(markup, /Sign in again/)
})
