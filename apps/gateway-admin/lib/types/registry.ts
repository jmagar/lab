// TypeScript types mirroring crates/lab-apis/src/mcpregistry/types.rs
// JSON field names follow serde rename annotations in the Rust source.

export const REGISTRY_META_KEY = 'io.modelcontextprotocol.registry/official'

export interface ListServersParams {
  search?: string
  limit?: number
  cursor?: string | null
  version?: string
  updated_since?: string
}

export interface ServerListResponse {
  servers: ServerResponse[]
  metadata: ListMetadata
}

export interface ListMetadata {
  count: number
  nextCursor?: string | null
}

export interface ServerResponse {
  server: ServerJSON
  _meta?: ResponseMeta | null
}

export interface ResponseMeta {
  'io.modelcontextprotocol.registry/official'?: RegistryExtensions | null
}

export interface RegistryExtensions {
  isLatest: boolean
  publishedAt: string
  status: 'active' | 'deprecated' | 'deleted'
  statusChangedAt: string
  statusMessage?: string | null
  updatedAt?: string | null
}

export interface ServerJSON {
  $schema?: string | null
  name: string
  title?: string | null
  description: string
  version: string
  packages: Package[]
  remotes: Transport[]
  repository?: Repository | null
  icons: Icon[]
  websiteUrl?: string | null
}

export interface Transport {
  type: 'streamable-http' | 'sse' | 'stdio' | string
  url?: string | null
  headers?: Header[] | null
  variables?: Record<string, unknown> | null
}

export interface Header {
  name: string
  value: string
}

export interface Package {
  registryType: string
  identifier: string
  version?: string | null
  transport: Transport
  runtimeHint?: string | null
  runtimeArguments?: RuntimeArgument[] | null
  packageArguments?: PackageArgument[] | null
  environmentVariables?: EnvironmentVariable[] | null
  fileSha256?: string | null
  registryBaseUrl?: string | null
}

export interface RuntimeArgument {
  name: string
  value?: string | null
}

export interface PackageArgument {
  value: string
  description?: string | null
  isRequired: boolean
}

export interface EnvironmentVariable {
  name: string
  description?: string | null
  required: boolean
}

export interface Repository {
  url: string
  source?: string | null
  id?: string | null
}

export interface Icon {
  src: string
  mimeType?: string | null
  sizes?: string[]
  theme?: string | null
}

export interface ValidationResult {
  valid: boolean
  issues: ValidationIssue[]
}

export interface ValidationIssue {
  field?: string | null
  message: string
  severity?: string | null
}

export class RegistryApiError extends Error implements RegistryError {
  status: number
  code?: string

  constructor(message: string, status: number, code?: string) {
    super(message)
    this.name = 'RegistryApiError'
    this.status = status
    this.code = code
  }
}

export interface RegistryError {
  message: string
  status: number
  code?: string
}
