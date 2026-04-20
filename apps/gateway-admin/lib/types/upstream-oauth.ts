export interface UpstreamEntry {
  name: string
}

export interface UpstreamOauthStatus {
  authenticated: boolean
  upstream: string
  expires_within_5m: boolean
}

export interface StartResponse {
  authorization_url: string
}

export interface ProbeResponse {
  upstream: string
  url: string
  oauth_discovered: boolean
  issuer?: string
  scopes?: string[]
  registration_strategy?: 'dynamic' | 'preregistered' | 'client_metadata_document'
}

export type OAuthConnectState =
  | { kind: 'idle' }
  | { kind: 'probing' }
  | { kind: 'discovered'; upstream: string; issuer?: string; scopes?: string[] }
  | { kind: 'authorizing'; upstream: string }
  | { kind: 'connected'; upstream: string; registration_strategy: string; scopes?: string[] }
  | { kind: 'error'; message: string }
