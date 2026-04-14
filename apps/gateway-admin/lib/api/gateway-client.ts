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

export const gatewayApi = {
  // List all gateways
  async list(): Promise<Gateway[]> {
    const response = await fetch(`${API_BASE}/gateways`)
    return handleResponse<Gateway[]>(response)
  },

  // Get a single gateway by ID
  async get(id: string): Promise<Gateway> {
    const response = await fetch(`${API_BASE}/gateways/${id}`)
    return handleResponse<Gateway>(response)
  },

  // Create a new gateway
  async create(input: CreateGatewayInput): Promise<Gateway> {
    const response = await fetch(`${API_BASE}/gateways`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
    })
    return handleResponse<Gateway>(response)
  },

  // Update an existing gateway
  async update(id: string, input: UpdateGatewayInput): Promise<Gateway> {
    const response = await fetch(`${API_BASE}/gateways/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
    })
    return handleResponse<Gateway>(response)
  },

  // Remove a gateway
  async remove(id: string): Promise<void> {
    const response = await fetch(`${API_BASE}/gateways/${id}`, {
      method: 'DELETE',
    })
    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Failed to delete gateway' }))
      throw new GatewayApiError(error.message, response.status, error.code)
    }
  },

  // Test gateway connection
  async test(id: string): Promise<TestGatewayResult> {
    const response = await fetch(`${API_BASE}/gateways/${id}/test`, {
      method: 'POST',
    })
    return handleResponse<TestGatewayResult>(response)
  },

  // Reload gateway (re-discover tools/resources/prompts)
  async reload(id: string): Promise<ReloadGatewayResult> {
    const response = await fetch(`${API_BASE}/gateways/${id}/reload`, {
      method: 'POST',
    })
    return handleResponse<ReloadGatewayResult>(response)
  },

  // Get exposure policy for a gateway
  async getExposurePolicy(id: string): Promise<ExposurePolicy> {
    const response = await fetch(`${API_BASE}/gateways/${id}/exposure`)
    return handleResponse<ExposurePolicy>(response)
  },

  // Update exposure policy
  async setExposurePolicy(id: string, policy: ExposurePolicy): Promise<ExposurePolicy> {
    const response = await fetch(`${API_BASE}/gateways/${id}/exposure`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(policy),
    })
    return handleResponse<ExposurePolicy>(response)
  },

  // Preview exposure policy (dry run)
  async previewExposurePolicy(
    id: string,
    patterns: string[]
  ): Promise<ExposurePolicyPreview> {
    const response = await fetch(`${API_BASE}/gateways/${id}/exposure/preview`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ patterns }),
    })
    return handleResponse<ExposurePolicyPreview>(response)
  },
}

export { GatewayApiError }
