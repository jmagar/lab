import { nodesUrl } from './gateway-config.ts'
import { gatewayHeaders } from './gateway-request.ts'

const USE_MOCK_DATA = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

export interface FleetDevice {
  node_id: string
  connected: boolean
  role: string
}

const MOCK_FLEET_DEVICES: FleetDevice[] = [
  { node_id: 'local', connected: true, role: 'master' },
  { node_id: 'node-alpha', connected: true, role: 'worker' },
  { node_id: 'node-beta', connected: false, role: 'worker' },
]

export async function fetchFleetDevices(signal?: AbortSignal): Promise<FleetDevice[]> {
  if (USE_MOCK_DATA) {
    signal?.throwIfAborted?.()
    return structuredClone(MOCK_FLEET_DEVICES)
  }

  const response = await fetch(nodesUrl(), {
    method: 'GET',
    headers: gatewayHeaders(),
    cache: 'no-store',
    credentials: 'include',
    signal,
  })

  if (!response.ok) {
    throw new Error(`Failed to fetch fleet devices: ${response.status}`)
  }

  return response.json() as Promise<FleetDevice[]>
}
