import type {
  Gateway,
  CreateGatewayInput,
  UpdateGatewayInput,
  TestGatewayResult,
  ReloadGatewayResult,
  ExposurePolicy,
  ExposurePolicyPreview,
} from '@/lib/types/gateway'

const API_BASE = process.env.NEXT_PUBLIC_API_URL || '/api'

class GatewayApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public code?: string
  ) {
    super(message)
    this.name = 'GatewayApiError'
  }
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'An error occurred' }))
    throw new GatewayApiError(
      error.message || 'An error occurred',
      response.status,
      error.code
    )
  }
  return response.json()
}

function gatewayPath(id: string, suffix = ''): string {
  return `${API_BASE}/gateways/${encodeURIComponent(id)}${suffix}`
}

export const gatewayApi = {
  // List all gateways
  async list(signal?: AbortSignal): Promise<Gateway[]> {
    const response = await fetch(`${API_BASE}/gateways`, { signal })
    return handleResponse<Gateway[]>(response)
  },

  // Get a single gateway by ID
  async get(id: string, signal?: AbortSignal): Promise<Gateway> {
    const response = await fetch(gatewayPath(id), { signal })
    return handleResponse<Gateway>(response)
  },

  // Create a new gateway
  async create(input: CreateGatewayInput, signal?: AbortSignal): Promise<Gateway> {
    const response = await fetch(`${API_BASE}/gateways`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
      signal,
    })
    return handleResponse<Gateway>(response)
  },

  // Update an existing gateway
  async update(id: string, input: UpdateGatewayInput, signal?: AbortSignal): Promise<Gateway> {
    const response = await fetch(gatewayPath(id), {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
      signal,
    })
    return handleResponse<Gateway>(response)
  },

  // Remove a gateway
  async remove(id: string, signal?: AbortSignal): Promise<void> {
    const response = await fetch(gatewayPath(id), {
      method: 'DELETE',
      signal,
    })
    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Failed to delete gateway' }))
      throw new GatewayApiError(error.message, response.status, error.code)
    }
  },

  // Test gateway connection
  async test(id: string, signal?: AbortSignal): Promise<TestGatewayResult> {
    const response = await fetch(gatewayPath(id, '/test'), {
      method: 'POST',
      signal,
    })
    return handleResponse<TestGatewayResult>(response)
  },

  // Reload gateway (re-discover tools/resources/prompts)
  async reload(id: string, signal?: AbortSignal): Promise<ReloadGatewayResult> {
    const response = await fetch(gatewayPath(id, '/reload'), {
      method: 'POST',
      signal,
    })
    return handleResponse<ReloadGatewayResult>(response)
  },

  // Get exposure policy for a gateway
  async getExposurePolicy(id: string, signal?: AbortSignal): Promise<ExposurePolicy> {
    const response = await fetch(gatewayPath(id, '/exposure'), { signal })
    return handleResponse<ExposurePolicy>(response)
  },

  // Update exposure policy
  async setExposurePolicy(id: string, policy: ExposurePolicy, signal?: AbortSignal): Promise<ExposurePolicy> {
    const response = await fetch(gatewayPath(id, '/exposure'), {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(policy),
      signal,
    })
    return handleResponse<ExposurePolicy>(response)
  },

  // Preview exposure policy (dry run)
  async previewExposurePolicy(
    id: string,
    patterns: string[],
    signal?: AbortSignal
  ): Promise<ExposurePolicyPreview> {
    const response = await fetch(gatewayPath(id, '/exposure/preview'), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ patterns }),
      signal,
    })
    return handleResponse<ExposurePolicyPreview>(response)
  },
}

export { GatewayApiError }
