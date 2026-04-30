import test from 'node:test'
import assert from 'node:assert/strict'

import {
  resolvePersistedChatLayoutState,
  shouldAutoBootstrapChat,
} from './admin-layout-client'
import {
  DEFAULT_CONFIG,
  writePersistedState,
} from './floating-chat-popover'

test('resolved persisted chat layout state carries open into bootstrap gating', () => {
  writePersistedState({
    open: true,
    config: { sendPageContext: true },
  })

  const state = resolvePersistedChatLayoutState()

  assert.deepEqual(state, {
    open: true,
    config: { sendPageContext: true },
  })
  assert.equal(shouldAutoBootstrapChat(state.open, false), true)
})

test('chat auto bootstrap remains disabled when closed outside the chat page', () => {
  writePersistedState({
    open: false,
    config: DEFAULT_CONFIG,
  })

  const state = resolvePersistedChatLayoutState()

  assert.deepEqual(state, {
    open: false,
    config: DEFAULT_CONFIG,
  })
  assert.equal(shouldAutoBootstrapChat(state.open, false), false)
  assert.equal(shouldAutoBootstrapChat(state.open, true), true)
})
