import type { BridgeEvent } from './types'

const encoder = new TextEncoder()

export function encodeSseEvent(event: BridgeEvent) {
  return encoder.encode(`id: ${event.seq}\ndata: ${JSON.stringify(event)}\n\n`)
}
