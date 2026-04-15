'use client'

import useSWR, { mutate } from 'swr'
import { gatewayApi } from '@/lib/api/gateway-client'
import { mockGateways, mockExposurePolicy, mockTestResult, mockReloadResult, mockExposurePolicyPreview } from '@/lib/api/mock-data'
import type { Gateway, CreateGatewayInput, UpdateGatewayInput, ExposurePolicy, TestGatewayResult, ReloadGatewayResult, ExposurePolicyPreview } from '@/lib/types/gateway'
import { useCallback } from 'react'

// Set NEXT_PUBLIC_MOCK_DATA=true to use mock data for development
const USE_MOCK_DATA = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

// Simulate network delay for mock data
const mockDelay = (ms: number = 500) => new Promise(resolve => setTimeout(resolve, ms))

function abortableMockDelay(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(new DOMException('Aborted', 'AbortError'))
      return
    }

    const timer = setTimeout(() => {
      signal?.removeEventListener('abort', onAbort)
      resolve()
    }, ms)

    const onAbort = () => {
      clearTimeout(timer)
      reject(new DOMException('Aborted', 'AbortError'))
    }

    signal?.addEventListener('abort', onAbort, { once: true })
  })
}

// Fetcher functions that handle mock/real data
const fetchGateways = async (): Promise<Gateway[]> => {
  if (USE_MOCK_DATA) {
    await mockDelay()
    return mockGateways
  }
  return gatewayApi.list()
}

const fetchGateway = async (id: string): Promise<Gateway> => {
  if (USE_MOCK_DATA) {
    await mockDelay()
    const gateway = mockGateways.find(g => g.id === id)
    if (!gateway) throw new Error('Gateway not found')
    return gateway
  }
  return gatewayApi.get(id)
}

const fetchExposurePolicy = async (id: string): Promise<ExposurePolicy> => {
  if (USE_MOCK_DATA) {
    await mockDelay()
    const gateway = mockGateways.find(g => g.id === id)
    if (!gateway) throw new Error('Gateway not found')
    return {
      mode: gateway.config.expose_tools ? 'allowlist' : 'expose_all',
      patterns: gateway.config.expose_tools || [],
    }
  }
  return gatewayApi.getExposurePolicy(id)
}

// SWR Keys
export const GATEWAYS_KEY = '/gateways'
export const gatewayKey = (id: string) => `/gateways/${id}`
export const exposurePolicyKey = (id: string) => `/gateways/${id}/exposure`

// Hooks
export function useGateways() {
  return useSWR<Gateway[]>(GATEWAYS_KEY, fetchGateways, {
    revalidateOnFocus: false,
  })
}

export function useGateway(id: string | null) {
  return useSWR<Gateway>(
    id ? gatewayKey(id) : null,
    id ? () => fetchGateway(id) : null,
    {
      revalidateOnFocus: false,
    }
  )
}

export function useExposurePolicy(id: string | null) {
  return useSWR<ExposurePolicy>(
    id ? exposurePolicyKey(id) : null,
    id ? () => fetchExposurePolicy(id) : null,
    {
      revalidateOnFocus: false,
    }
  )
}

// Mutation hooks
export function useGatewayMutations() {
  const createGateway = useCallback(async (input: CreateGatewayInput): Promise<Gateway> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      const newGateway: Gateway = {
        id: `gw-${Date.now()}`,
        name: input.name,
        transport: input.transport,
        config: input.config,
        status: {
          healthy: false,
          connected: false,
          discovered_tool_count: 0,
          exposed_tool_count: 0,
        },
        discovery: { tools: [], resources: [], prompts: [] },
        warnings: [],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }
      await mutate(GATEWAYS_KEY, (current: Gateway[] = []) => [...current, newGateway], false)
      return newGateway
    }
    const gateway = await gatewayApi.create(input)
    await mutate(GATEWAYS_KEY)
    return gateway
  }, [])

  const updateGateway = useCallback(async (id: string, input: UpdateGatewayInput): Promise<Gateway> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      const gateway = mockGateways.find(g => g.id === id)
      if (!gateway) throw new Error('Gateway not found')
      const updated = { ...gateway, ...input, updated_at: new Date().toISOString() }
      await mutate(gatewayKey(id), updated, false)
      await mutate(GATEWAYS_KEY)
      return updated
    }
    const gateway = await gatewayApi.update(id, input)
    await mutate(gatewayKey(id))
    await mutate(GATEWAYS_KEY)
    return gateway
  }, [])

  const removeGateway = useCallback(async (id: string): Promise<void> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      await mutate(GATEWAYS_KEY, (current: Gateway[] = []) => current.filter(g => g.id !== id), false)
      return
    }
    await gatewayApi.remove(id)
    await mutate(GATEWAYS_KEY)
  }, [])

  const testGateway = useCallback(async (id: string): Promise<TestGatewayResult> => {
    if (USE_MOCK_DATA) {
      await mockDelay(1500) // Longer delay for test
      const gateway = mockGateways.find(g => g.id === id)
      if (!gateway) throw new Error('Gateway not found')
      if (!gateway.status.healthy) {
        return {
          success: false,
          message: 'Connection failed',
          error: gateway.status.last_error,
        }
      }
      return mockTestResult
    }
    return await gatewayApi.test(id)
  }, [])

  const reloadGateway = useCallback(async (id: string): Promise<ReloadGatewayResult> => {
    if (USE_MOCK_DATA) {
      await mockDelay(2000) // Longer delay for reload
      return mockReloadResult
    }
    const result = await gatewayApi.reload(id)
    await mutate(gatewayKey(id))
    await mutate(GATEWAYS_KEY)
    return result
  }, [])

  const setExposurePolicy = useCallback(async (id: string, policy: ExposurePolicy): Promise<ExposurePolicy> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      await mutate(exposurePolicyKey(id), policy, false)
      await mutate(gatewayKey(id))
      return policy
    }
    const result = await gatewayApi.setExposurePolicy(id, policy)
    await mutate(exposurePolicyKey(id))
    await mutate(gatewayKey(id))
    return result
  }, [])

  const previewExposurePolicy = useCallback(async (id: string, patterns: string[], signal?: AbortSignal): Promise<ExposurePolicyPreview> => {
    if (USE_MOCK_DATA) {
      await abortableMockDelay(300, signal)
      const gateway = mockGateways.find(g => g.id === id)
      if (!gateway) throw new Error('Gateway not found')

      if (patterns.length === 0) {
        return {
          matched_tools: gateway.discovery.tools.map((tool) => ({
            name: tool.name,
            matched_by: '*',
          })),
          unmatched_patterns: [],
          filtered_tools: [],
          exposed_count: gateway.discovery.tools.length,
          filtered_count: 0,
        }
      }
      
      const matchedTools: Array<{ name: string; matched_by: string }> = []
      const filteredTools: string[] = []
      const usedPatterns = new Set<string>()

      for (const tool of gateway.discovery.tools) {
        let matched = false
        for (const pattern of patterns) {
          if (pattern === '*') {
            matchedTools.push({ name: tool.name, matched_by: pattern })
            usedPatterns.add(pattern)
            matched = true
            break
          } else if (pattern.endsWith('*')) {
            const prefix = pattern.slice(0, -1)
            if (tool.name.startsWith(prefix)) {
              matchedTools.push({ name: tool.name, matched_by: pattern })
              usedPatterns.add(pattern)
              matched = true
              break
            }
          } else if (tool.name === pattern) {
            matchedTools.push({ name: tool.name, matched_by: pattern })
            usedPatterns.add(pattern)
            matched = true
            break
          }
        }
        if (!matched) {
          filteredTools.push(tool.name)
        }
      }

      const unmatchedPatterns = patterns.filter(p => !usedPatterns.has(p))

      return {
        matched_tools: matchedTools,
        unmatched_patterns: unmatchedPatterns,
        filtered_tools: filteredTools,
        exposed_count: matchedTools.length,
        filtered_count: filteredTools.length,
      }
    }
    return gatewayApi.previewExposurePolicy(id, patterns, signal)
  }, [])

  return {
    createGateway,
    updateGateway,
    removeGateway,
    testGateway,
    reloadGateway,
    setExposurePolicy,
    previewExposurePolicy,
  }
}
