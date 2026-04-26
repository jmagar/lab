import test from 'node:test'
import assert from 'node:assert/strict'

import { normalizeProviderEvent } from './normalize'
import type { ProviderRuntimeEvent } from './types'

test('normalizeProviderEvent maps Stage 3 session updates into browser ACP event kinds', () => {
  const [planEvent] = normalizeProviderEvent(
    'session-1',
    'codex',
    {
      type: 'session_notification',
      notification: {
        update: {
          sessionUpdate: 'plan',
          entries: [{ content: 'Inspect repo', priority: 'high', status: 'in_progress' }],
        },
      },
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(planEvent.kind, 'plan')
  assert.equal(planEvent.title, 'Execution plan updated')
  assert.deepEqual(planEvent.plan, [{ content: 'Inspect repo', priority: 'high', status: 'in_progress' }])

  const [commandsEvent] = normalizeProviderEvent(
    'session-1',
    'codex',
    {
      type: 'session_notification',
      notification: {
        update: {
          sessionUpdate: 'available_commands_update',
          availableCommands: [{ name: 'cancel', description: 'Cancel the active prompt' }],
        },
      },
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(commandsEvent.kind, 'commands')
  assert.equal(commandsEvent.commands?.[0]?.name, 'cancel')

  const [modeEvent] = normalizeProviderEvent(
    'session-1',
    'codex',
    {
      type: 'session_notification',
      notification: {
        update: {
          sessionUpdate: 'current_mode_update',
          slug: 'review',
          name: 'Review',
        },
      },
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(modeEvent.kind, 'mode')
  assert.equal(modeEvent.title, 'Agent mode updated')
  assert.equal((modeEvent.currentMode as { slug?: string }).slug, 'review')

  const [configEvent] = normalizeProviderEvent(
    'session-1',
    'codex',
    {
      type: 'session_notification',
      notification: {
        update: {
          sessionUpdate: 'config_option_update',
          configOptions: [{ key: 'temperature', value: 0.2 }],
        },
      },
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(configEvent.kind, 'config')
  assert.equal(configEvent.title, 'Configuration options updated')
  assert.deepEqual((configEvent.configUpdate as { configOptions: unknown[] }).configOptions, [{ key: 'temperature', value: 0.2 }])

  const [sessionInfoEvent] = normalizeProviderEvent(
    'session-1',
    'codex',
    {
      type: 'session_notification',
      notification: {
        update: {
          sessionUpdate: 'session_info_update',
          title: 'Renamed session',
        },
      },
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(sessionInfoEvent.kind, 'session.info')
  assert.equal(sessionInfoEvent.sessionInfo?.title, 'Renamed session')

  const [usageEvent] = normalizeProviderEvent(
    'session-1',
    'codex',
    {
      type: 'session_notification',
      notification: {
        update: {
          sessionUpdate: 'usage_update',
          used: 12,
          size: 256,
        },
      },
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(usageEvent.kind, 'usage')
  assert.equal(usageEvent.usage?.used, 12)
  assert.equal(usageEvent.usage?.size, 256)

  const [fallbackEvent] = normalizeProviderEvent(
    'session-1',
    'codex',
    {
      type: 'session_notification',
      notification: {
        update: {
          sessionUpdate: 'unrecognized_update',
          rawValue: true,
        },
      },
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(fallbackEvent.kind, 'debug')
  assert.equal(fallbackEvent.title, 'Unhandled session update')
  assert.deepEqual(fallbackEvent.raw, { sessionUpdate: 'unrecognized_update', rawValue: true })
})

test('normalizeProviderEvent maps permission, prompt, exit, and error events into the browser contract', () => {
  const [permissionRequested] = normalizeProviderEvent(
    'session-9',
    'codex',
    {
      type: 'permission_request',
      request: {
        toolCall: {
          toolCallId: 'tool-1',
          title: 'Read ~/.ssh/config',
        },
        options: [
          { optionId: 'allow_once', name: 'Allow once', kind: 'allow_once' },
          { optionId: 'deny', name: 'Deny', kind: 'deny' },
        ],
      },
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(permissionRequested.kind, 'permission.requested')
  assert.equal(permissionRequested.toolCallId, 'tool-1')
  assert.equal(permissionRequested.status, 'requested')
  assert.deepEqual(permissionRequested.permissionOptions, [
    { optionId: 'allow_once', name: 'Allow once', kind: 'allow_once' },
    { optionId: 'deny', name: 'Deny', kind: 'deny' },
  ])

  const [permissionResolved] = normalizeProviderEvent(
    'session-9',
    'codex',
    {
      type: 'permission_resolved',
      request: {
        toolCall: {
          toolCallId: 'tool-1',
          title: 'Read ~/.ssh/config',
        },
        options: [],
      },
      selectedOptionId: 'allow_once',
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(permissionResolved.kind, 'permission.resolved')
  assert.equal(permissionResolved.status, 'resolved')
  assert.equal(permissionResolved.permissionSelection, 'allow_once')

  const [promptStarted] = normalizeProviderEvent(
    'session-9',
    'codex',
    { type: 'prompt_started', prompt: 'Summarize Stage 3 browser coverage' },
  )
  assert.equal(promptStarted.kind, 'status')
  assert.equal(promptStarted.title, 'Prompt started')
  assert.equal(promptStarted.status, 'running')
  assert.equal(promptStarted.text, 'Summarize Stage 3 browser coverage')

  const [promptCancelled] = normalizeProviderEvent(
    'session-9',
    'codex',
    {
      type: 'prompt_response',
      response: {
        stopReason: 'cancelled',
      },
    } as unknown as ProviderRuntimeEvent,
  )
  assert.equal(promptCancelled.kind, 'status')
  assert.equal(promptCancelled.title, 'Prompt completed')
  assert.equal(promptCancelled.status, 'cancelled')
  assert.equal(promptCancelled.promptStopReason, 'cancelled')

  const [processExit] = normalizeProviderEvent(
    'session-9',
    'codex',
    { type: 'process_exit', code: 1, signal: 'SIGTERM' },
  )
  assert.equal(processExit.kind, 'status')
  assert.equal(processExit.title, 'Provider process exited')
  assert.equal(processExit.status, 'failed')
  assert.match(processExit.text ?? '', /code=1 signal=SIGTERM/)

  const [providerError] = normalizeProviderEvent(
    'session-9',
    'codex',
    { type: 'error', message: 'Prompt failed', raw: { retryable: false } },
  )
  assert.equal(providerError.kind, 'error')
  assert.equal(providerError.title, 'Provider error')
  assert.equal(providerError.status, 'failed')
  assert.equal(providerError.text, 'Prompt failed')
  assert.deepEqual(providerError.raw, { retryable: false })
})
