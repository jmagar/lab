'use client'

import useSWR, { mutate } from 'swr'
import { gatewayApi } from '@/lib/api/gateway-client'
import {
  getMockGatewayFallback,
  getMockGatewaysFallback,
  getMockServiceActionsFallback,
  getMockServiceConfigFallback,
  getMockSupportedServicesFallback,
} from '@/lib/api/mock-fallback'
import { EXPOSE_NONE_PATTERN } from '@/lib/api/tool-exposure-draft'
import { getMockGatewayOverride, setMockGatewayOverride } from '@/lib/api/mock-gateway-overrides'
import {
  mockGateways,
  mockReloadResult,
  mockTestResult,
} from '@/lib/api/mock-data'
import type {
  Gateway,
  CreateGatewayInput,
  UpdateGatewayInput,
  ExposurePolicy,
  TestGatewayResult,
  ReloadGatewayResult,
  ExposurePolicyPreview,
  ServiceConfig,
  ServiceAction,
  SupportedService,
} from '@/lib/types/gateway'
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
    return getMockGatewaysFallback()
  }
  return gatewayApi.list()
}

const fetchGateway = async (id: string): Promise<Gateway> => {
  if (USE_MOCK_DATA) {
    await mockDelay()
    const gateway = getMockGatewayFallback(id)
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

const fetchSupportedServices = async (): Promise<SupportedService[]> => {
  if (USE_MOCK_DATA) {
    await mockDelay()
    return getMockSupportedServicesFallback()
  }
  return gatewayApi.supportedServices()
}

const fetchServiceConfig = async (service: string): Promise<ServiceConfig> => {
  if (USE_MOCK_DATA) {
    await mockDelay()
    return getMockServiceConfigFallback(service)
  }
  return gatewayApi.getServiceConfig(service)
}

const fetchServiceActions = async (service: string): Promise<ServiceAction[]> => {
  if (USE_MOCK_DATA) {
    await mockDelay()
    return getMockServiceActionsFallback(service)
  }
  return gatewayApi.serviceActions(service)
}

// SWR Keys
export const GATEWAYS_KEY = '/gateways'
export const gatewayKey = (id: string) => `/gateways/${id}`
export const exposurePolicyKey = (id: string) => `/gateways/${id}/exposure`
export const SUPPORTED_SERVICES_KEY = '/gateway-supported-services'
export const serviceConfigKey = (service: string) => `/gateway-service-config/${service}`
export const serviceActionsKey = (service: string) => `/gateway-service-actions/${service}`

async function refreshGatewayCache(id?: string, extraKeys: string[] = []) {
  const keys = [GATEWAYS_KEY, ...(id ? [gatewayKey(id)] : []), ...extraKeys]
  await Promise.all(keys.map((key) => mutate(key)))
}

// Hooks
export function useGateways() {
  return useSWR<Gateway[]>(GATEWAYS_KEY, fetchGateways, {
    revalidateOnFocus: false,
    fallbackData: USE_MOCK_DATA ? getMockGatewaysFallback() : undefined,
    revalidateOnMount: !USE_MOCK_DATA,
  })
}

export function useGateway(id: string | null) {
  const fallbackGateway = USE_MOCK_DATA && id ? getMockGatewayFallback(id) : undefined

  return useSWR<Gateway>(
    id ? gatewayKey(id) : null,
    id ? () => fetchGateway(id) : null,
    {
      revalidateOnFocus: false,
      fallbackData: fallbackGateway,
      revalidateOnMount: !USE_MOCK_DATA || fallbackGateway === undefined,
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

export function useSupportedServices() {
  return useSWR<SupportedService[]>(SUPPORTED_SERVICES_KEY, fetchSupportedServices, {
    revalidateOnFocus: false,
    fallbackData: USE_MOCK_DATA ? getMockSupportedServicesFallback() : undefined,
    revalidateOnMount: !USE_MOCK_DATA,
  })
}

export function useServiceConfig(service: string | null) {
  return useSWR<ServiceConfig>(
    service ? serviceConfigKey(service) : null,
    service ? () => fetchServiceConfig(service) : null,
    {
      revalidateOnFocus: false,
      fallbackData: USE_MOCK_DATA && service ? getMockServiceConfigFallback(service) : undefined,
      revalidateOnMount: !USE_MOCK_DATA,
    }
  )
}

export function useServiceActions(service: string | null) {
  return useSWR<ServiceAction[]>(
    service ? serviceActionsKey(service) : null,
    service ? () => fetchServiceActions(service) : null,
    {
      revalidateOnFocus: false,
      fallbackData: USE_MOCK_DATA && service ? getMockServiceActionsFallback(service) : undefined,
      revalidateOnMount: !USE_MOCK_DATA,
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
          discovered_resource_count: 0,
          exposed_resource_count: 0,
          discovered_prompt_count: 0,
          exposed_prompt_count: 0,
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
    await refreshGatewayCache(gateway.id)
    return gateway
  }, [])

  const updateGateway = useCallback(async (id: string, input: UpdateGatewayInput): Promise<Gateway> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      const gateway = mockGateways.find(g => g.id === id)
      if (!gateway) throw new Error('Gateway not found')
      const updated = {
        ...gateway,
        ...input,
        config: {
          ...gateway.config,
          ...input.config,
        },
        updated_at: new Date().toISOString(),
      }
      if (input.config?.proxy_resources !== undefined) {
        setMockGatewayOverride(id, { proxyResources: input.config.proxy_resources })
      }
      await mutate(gatewayKey(id), updated, false)
      await mutate(GATEWAYS_KEY)
      return updated
    }
    const gateway = await gatewayApi.update(id, input)
    await refreshGatewayCache(id)
    return gateway
  }, [])

  const removeGateway = useCallback(async (id: string): Promise<void> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      await mutate(GATEWAYS_KEY, (current: Gateway[] = []) => current.filter(g => g.id !== id), false)
      return
    }
    await gatewayApi.remove(id)
    await refreshGatewayCache()
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
    await refreshGatewayCache(id)
    return result
  }, [])

  const setExposurePolicy = useCallback(async (id: string, policy: ExposurePolicy): Promise<ExposurePolicy> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      setMockGatewayOverride(id, { exposurePolicy: policy })
      const updatedGateway = getMockGatewayFallback(id)
      if (!updatedGateway) {
        throw new Error('Gateway not found')
      }

      await mutate(
        gatewayKey(id),
        updatedGateway,
        false,
      )
      await mutate(
        GATEWAYS_KEY,
        (current: Gateway[] = []) =>
          current.map((gateway) => (gateway.id === id ? updatedGateway : gateway)),
        false,
      )
      await mutate(exposurePolicyKey(id), policy, false)
      return policy
    }
    const result = await gatewayApi.setExposurePolicy(id, policy)
    await refreshGatewayCache(id, [exposurePolicyKey(id)])
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

  const saveServiceConfig = useCallback(async (service: string, values: Record<string, string>): Promise<ServiceConfig> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      const fields = Object.entries(values).map(([name, value]) => ({
        name,
        present: value.length > 0,
        secret: name.includes('TOKEN') || name.includes('KEY') || name.includes('PASSWORD'),
        value_preview: name.includes('TOKEN') || name.includes('KEY') || name.includes('PASSWORD') ? null : value,
      }))
      const result = { service, configured: fields.length > 0, fields }
      await mutate(serviceConfigKey(service), result, false)
      return result
    }
    const result = await gatewayApi.setServiceConfig(service, values)
    await refreshGatewayCache(undefined, [serviceConfigKey(service)])
    return result
  }, [])

  const enableVirtualServer = useCallback(async (id: string): Promise<Gateway> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      const gateway = mockGateways.find((item) => item.id === id)
      if (!gateway) throw new Error('Gateway not found')
      const result = { ...gateway, enabled: true }
      await mutate(gatewayKey(id), result, false)
      await mutate(GATEWAYS_KEY)
      return result
    }
    const result = await gatewayApi.enableVirtualServer(id)
    await refreshGatewayCache(id)
    return result
  }, [])

  const disableVirtualServer = useCallback(async (id: string): Promise<Gateway> => {
    if (USE_MOCK_DATA) {
      await mockDelay()
      const gateway = mockGateways.find((item) => item.id === id)
      if (!gateway) throw new Error('Gateway not found')
      const result = { ...gateway, enabled: false }
      await mutate(gatewayKey(id), result, false)
      await mutate(GATEWAYS_KEY)
      return result
    }
    const result = await gatewayApi.disableVirtualServer(id)
    await refreshGatewayCache(id)
    return result
  }, [])

  const setVirtualServerSurface = useCallback(
    async (id: string, surface: 'cli' | 'api' | 'mcp' | 'webui', enabled: boolean): Promise<Gateway> => {
      if (USE_MOCK_DATA) {
        await mockDelay()
        const gateway = mockGateways.find((item) => item.id === id)
        if (!gateway) throw new Error('Gateway not found')
        const result = {
          ...gateway,
          surfaces: gateway.surfaces
            ? {
                ...gateway.surfaces,
                [surface]: { ...gateway.surfaces[surface], enabled },
              }
            : gateway.surfaces,
        }
        await mutate(gatewayKey(id), result, false)
        await mutate(GATEWAYS_KEY)
        return result
      }
      const result = await gatewayApi.setVirtualServerSurface(id, surface, enabled)
      await refreshGatewayCache(id)
      return result
    },
    [],
  )

  return {
    createGateway,
    updateGateway,
    removeGateway,
    testGateway,
    reloadGateway,
    setExposurePolicy,
    previewExposurePolicy,
    saveServiceConfig,
    enableVirtualServer,
    disableVirtualServer,
    setVirtualServerSurface,
  }
}
