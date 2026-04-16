import type {
  Gateway,
  ServiceAction,
  ServiceConfig,
  SupportedService,
} from '../types/gateway.ts'
import {
  mockGateways,
  mockServiceActions,
  mockServiceConfigs,
  mockSupportedServices,
} from './mock-data.ts'
import { applyMockGatewayOverride, getMockGatewayOverride } from './mock-gateway-overrides.ts'

function cloneValue<T>(value: T): T {
  return structuredClone(value)
}

export function getMockGatewaysFallback(): Gateway[] {
  return cloneValue(mockGateways).map((gateway) =>
    applyMockGatewayOverride(gateway, getMockGatewayOverride(gateway.id)),
  )
}

export function getMockGatewayFallback(id: string): Gateway | undefined {
  const gateway = mockGateways.find((item) => item.id === id)
  return gateway ? applyMockGatewayOverride(cloneValue(gateway), getMockGatewayOverride(id)) : undefined
}

export function getMockSupportedServicesFallback(): SupportedService[] {
  return cloneValue(mockSupportedServices)
}

export function getMockServiceConfigFallback(service: string): ServiceConfig {
  return cloneValue(mockServiceConfigs[service] ?? { service, configured: false, fields: [] })
}

export function getMockServiceActionsFallback(service: string): ServiceAction[] {
  return cloneValue(mockServiceActions[service] ?? [])
}
