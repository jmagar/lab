import { nodesUrl, nodeDetailUrl } from './gateway-config.ts'
import { gatewayHeaders } from './gateway-request.ts'

const USE_MOCK_DATA = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

export interface FleetNodeStatus {
  node_id: string
  connected: boolean
  cpu_percent: number | null
  memory_used_bytes: number | null
  storage_used_bytes: number | null
  os?: string | null
  ips?: string[]
  version?: string | null
  uptime_seconds?: number | null
  role?: string | null
  health?: 'healthy' | 'needs_attention' | 'disconnected'
  doctor_issues?: string[]
  active_claude_sessions?: number
  active_codex_sessions?: number
  gpu_usage_percent?: number | null
  cores?: number | null
  cpu_clock_mhz?: number | null
  cpu_temp_c?: number | null
  gpu_temp_c?: number | null
  fan_speeds_rpm?: number[]
  docker?: {
    installed?: boolean
    running?: number
    stopped?: number
  }
  zfs?: Array<{
    name: string
    size_gib: number | null
    used_gib: number | null
    available_gib: number | null
  }>
}

export interface FleetDevice {
  node_id: string
  connected: boolean
  role: string
  status?: FleetNodeStatus | null
  metadata?: {
    discovered_configs?: Array<{
      path: string
      version: string
    }>
  } | null
  discovered_config_count?: number
  log_count?: number
  version?: string | null
}

export interface FleetNodeDetail extends FleetDevice {
  status: FleetNodeStatus | null
}

const MOCK_FLEET_DEVICES: FleetDevice[] = [
  {
    node_id: 'local',
    connected: true,
    role: 'master',
    status: {
      node_id: 'local',
      connected: true,
      cpu_percent: 9.8,
      memory_used_bytes: 6_872_000_000,
      storage_used_bytes: 120_400_000_000,
      os: 'debian-12',
    ips: ['192.168.1.11', '10.0.0.11'],
      version: '0.7.4',
      uptime_seconds: 14_320,
      health: 'healthy',
      doctor_issues: [],
      active_claude_sessions: 2,
      active_codex_sessions: 0,
      gpu_usage_percent: 18.2,
      cores: 6,
      cpu_clock_mhz: 2_200,
      cpu_temp_c: 48,
      gpu_temp_c: null,
      fan_speeds_rpm: [2100, 1900],
      docker: { installed: true, running: 8, stopped: 1 },
      zfs: [{
        name: 'tank',
        size_gib: 800,
        used_gib: 245.7,
        available_gib: 554.3,
      }],
    },
    discovered_config_count: 8,
    log_count: 42,
  },
  {
    node_id: 'node-alpha',
    connected: true,
    role: 'worker',
    status: {
      node_id: 'node-alpha',
      connected: true,
      cpu_percent: 21.5,
      memory_used_bytes: 3_210_000_000,
      storage_used_bytes: 88_120_000_000,
      os: 'ubuntu-24.04',
      ips: ['192.168.1.25'],
      version: '0.7.3',
      uptime_seconds: 8_900,
      health: 'needs_attention',
      doctor_issues: ['high_load_detected', 'service_restart_pending'],
      active_claude_sessions: 1,
      active_codex_sessions: 1,
      gpu_usage_percent: 48,
      cores: 4,
      cpu_clock_mhz: 2_600,
      cpu_temp_c: 74,
      gpu_temp_c: 62,
      fan_speeds_rpm: [1850],
      docker: { installed: true, running: 4, stopped: 2 },
    },
    discovered_config_count: 2,
    log_count: 15,
  },
  {
    node_id: 'node-beta',
    connected: false,
    role: 'worker',
    status: {
      node_id: 'node-beta',
      connected: false,
      cpu_percent: null,
      memory_used_bytes: null,
      storage_used_bytes: null,
      os: 'ubuntu-24.04',
      ips: ['192.168.1.32'],
      version: '0.7.1',
      uptime_seconds: null,
      health: 'disconnected',
      doctor_issues: ['node_unreachable'],
      active_claude_sessions: 0,
      active_codex_sessions: 0,
      cores: null,
      docker: { installed: false, running: 0, stopped: 0 },
    },
    discovered_config_count: 1,
    log_count: 0,
  },
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

export async function fetchFleetNode(nodeId: string, signal?: AbortSignal): Promise<FleetNodeDetail> {
  if (USE_MOCK_DATA) {
    signal?.throwIfAborted?.()
    const device = MOCK_FLEET_DEVICES.find((item) => item.node_id === nodeId)
    if (!device) {
      throw new Error(`Unknown mock node: ${nodeId}`)
    }
    return {
      ...device,
      status: device.status ?? null,
      discovered_config_count: device.discovered_config_count ?? 0,
      log_count: device.log_count ?? 0,
      metadata: {
        discovered_configs: device.discovered_config_count
          ? Array.from({ length: device.discovered_config_count }).map((_, index) => ({
            path: `/cfg/config-${index + 1}.toml`,
            version: '1',
          }))
          : [],
      },
    } as FleetNodeDetail
  }

  const response = await fetch(nodeDetailUrl(nodeId), {
    method: 'GET',
    headers: gatewayHeaders(),
    cache: 'no-store',
    credentials: 'include',
    signal,
  })

  if (!response.ok) {
    throw new Error(`Failed to fetch node details: ${response.status}`)
  }

  return response.json() as Promise<FleetNodeDetail>
}
