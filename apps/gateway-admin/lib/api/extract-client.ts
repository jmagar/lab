import { extractActionUrl } from './gateway-config.ts'
import { gatewayRequestInit } from './gateway-request.ts'

export interface ExtractRuntimeMeta {
  container_name?: string
  image?: string | null
}

export interface ExtractCredential {
  service: string
  url: string | null
  secret: string | null
  env_field: string
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

export class ExtractApiError extends Error {
  status: number
  code?: string

  constructor(message: string, status: number, code?: string) {
    super(message)
    this.name = 'ExtractApiError'
    this.status = status
    this.code = code
  }
}

function isAbortError(error: unknown): boolean {
  return error instanceof DOMException
    ? error.name === 'AbortError'
    : error instanceof Error && error.name === 'AbortError'
}

async function parseExtractResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'An error occurred' }))
    throw new ExtractApiError(
      error.message || 'An error occurred',
      response.status,
      error.kind || error.code,
    )
  }

  return response.json()
}

async function extractAction<T>(
  action: string,
  params: object,
  signal?: AbortSignal,
): Promise<T> {
  let response: Response
  try {
    response = await fetch(
      extractActionUrl(),
      gatewayRequestInit(action, params, undefined, signal),
    )
  } catch (error) {
    if (isAbortError(error)) {
      throw error
    }
    const message = error instanceof Error ? error.message : 'unknown network error'
    throw new ExtractApiError(
      `Extract backend action \`${action}\` failed before a response was received: ${message}`,
      502,
      'backend_unreachable',
    )
  }

  return parseExtractResponse<T>(response)
}

export const extractApi = {
  async scan(uri?: string, signal?: AbortSignal): Promise<ExtractReport> {
    const params = uri === undefined ? {} : { uri }
    return extractAction<ExtractReport>('scan', params, signal)
  },
}
