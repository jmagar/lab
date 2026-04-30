import test from 'node:test'
import assert from 'node:assert/strict'

import { requestAcpSubscribeTicket } from './fetch'

test('requestAcpSubscribeTicket posts to the session ticket endpoint', async () => {
  const calls: Array<{ path: string; init?: RequestInit }> = []
  const fetchAcp = async (path: string, init?: RequestInit) => {
    calls.push({ path, init })
    return new Response(JSON.stringify({ ticket: 'ticket-1' }), { status: 200 })
  }

  const signal = new AbortController().signal
  const ticket = await requestAcpSubscribeTicket(fetchAcp, 'session 1', signal)

  assert.equal(ticket, 'ticket-1')
  assert.equal(calls.length, 1)
  assert.equal(calls[0].path, '/sessions/session%201/subscribe_ticket')
  assert.equal(calls[0].init?.method, 'POST')
  assert.equal(calls[0].init?.signal, signal)
})

test('requestAcpSubscribeTicket rejects non-ok responses', async () => {
  const fetchAcp = async () => new Response(JSON.stringify({ kind: 'auth_failed' }), { status: 401 })

  await assert.rejects(
    () => requestAcpSubscribeTicket(fetchAcp, 'session-1'),
    /failed with 401/,
  )
})

test('requestAcpSubscribeTicket rejects malformed bodies', async () => {
  const fetchAcp = async () => new Response(JSON.stringify({ ticket: '' }), { status: 200 })

  await assert.rejects(
    () => requestAcpSubscribeTicket(fetchAcp, 'session-1'),
    /missing ticket/,
  )
})
