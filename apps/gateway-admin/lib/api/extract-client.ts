import { extractActionUrl } from './gateway-config.ts'
import { performServiceAction, type ServiceActionError } from './service-action-client.ts'

export interface ExtractRuntimeMeta {
  container_name?: string
  image?: string | null
}

export interface ExtractCredential {
  service: string
  url: string | null
  env_field: string
  secret_present: boolean
  source_host?: string
  probe_host?: string
  runtime?: ExtractRuntimeMeta
  url_verified?: boolean
}

export interface ExtractWarning {
  service?: string
  host?: string
  runtime?: ExtractRuntimeMeta
  message: string
}

export interface ExtractSshUri {
  host: string
  path: string
}

export interface ExtractTarget {
  mode: 'fleet' | 'targeted'
  uri?: string | ExtractSshUri
}

export interface ExtractReport {
  target: ExtractTarget
  found: string[]
  creds: ExtractCredential[]
  warnings: ExtractWarning[]
}

export class ExtractApiError extends Error implements ServiceActionError {
  status: number
  code?: string

  constructor(message: string, status: number, code?: string) {
    super(message)
    this.name = 'ExtractApiError'
    this.status = status
    this.code = code
  }
}

async function extractAction<T>(
  action: string,
  params: object,
  signal?: AbortSignal,
): Promise<T> {
  return performServiceAction<T, ExtractApiError>({
    action,
    params,
    signal,
    serviceLabel: 'Extract',
    url: extractActionUrl(),
    createError: (message, status, code) => new ExtractApiError(message, status, code),
  })
}

export const extractApi = {
  async scan(uri?: string, signal?: AbortSignal): Promise<ExtractReport> {
    const params = uri === undefined ? { redact_secrets: true } : { uri, redact_secrets: true }
    return extractAction<ExtractReport>('scan', params, signal)
  },
}
