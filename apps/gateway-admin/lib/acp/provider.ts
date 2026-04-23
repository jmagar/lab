import type { ProviderHealth, ProviderRuntimeEvent, StartSessionInput, StartSessionResult } from './types'

export type ProviderEventHandler = (event: ProviderRuntimeEvent) => void

export interface AcpProviderHandle {
  providerSessionId: string
}

export interface AcpProvider {
  readonly kind: string
  health(): Promise<ProviderHealth>
  startSession(input: StartSessionInput, onEvent: ProviderEventHandler): Promise<{
    handle: AcpProviderHandle
    result: StartSessionResult
  }>
  promptSession(handle: AcpProviderHandle, prompt: string): Promise<void>
  cancelSession(handle: AcpProviderHandle): Promise<void>
  shutdownSession(handle: AcpProviderHandle): Promise<void>
  listSessions(): Promise<Array<{ providerSessionId: string }>>
  loadSession?(
    providerSessionId: string,
    input: StartSessionInput,
    onEvent: ProviderEventHandler,
  ): Promise<{
    handle: AcpProviderHandle
    result: StartSessionResult
  }>
}
