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
