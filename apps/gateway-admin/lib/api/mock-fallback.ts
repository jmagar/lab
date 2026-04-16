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

function cloneValue<T>(value: T): T {
  return structuredClone(value)
}

export function getMockGatewaysFallback(): Gateway[] {
  return cloneValue(mockGateways)
}

export function getMockGatewayFallback(id: string): Gateway | undefined {
  const gateway = mockGateways.find((item) => item.id === id)
  return gateway ? cloneValue(gateway) : undefined
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
