// TypeScript wrapper over the lab-bg3e.3 setup dispatch service.
//
// Mirrors crates/lab/src/dispatch/setup/catalog.rs. All actions go through
// POST /v1/setup with { action, params } shape (same as MCP).
//
// Each action returns the typed response directly; transport errors throw
// SetupApiError with the stable kind tag from docs/ERRORS.md.

import { setupActionUrl } from './gateway-config.ts'
import { performServiceAction, type ServiceActionError } from './service-action-client.ts'

export class SetupApiError extends Error implements ServiceActionError {
  status: number
  code?: string

  constructor(message: string, status: number, code?: string) {
    super(message)
    this.name = 'SetupApiError'
    this.status = status
    this.code = code
  }
}

async function setupAction<T>(
  action: string,
  params: Record<string, unknown> = {},
  signal?: AbortSignal,
): Promise<T> {
  return performServiceAction<T, SetupApiError>({
    action,
    params,
    signal,
    serviceLabel: 'Setup',
    url: setupActionUrl(),
    createError: (message, status, code) => new SetupApiError(message, status, code),
  })
}

// ─── State machine ──────────────────────────────────────────────────────

export type SetupStateKind =
  | 'uninitialized'
  | 'config_missing'
  | 'partially_configured'
  | 'health_checking'
  | 'ready'

export interface SetupState {
  kind: SetupStateKind
  envars?: string[]
  missing?: string[]
  services?: string[]
}

export interface SetupSnapshot {
  first_run: boolean
  env_path: string
  draft_path: string
  last_completed_step: number
  draft_stale: boolean
  has_draft: boolean
  state: SetupState
}

// ─── UiSchema projection ────────────────────────────────────────────────

export type FieldKindKey =
  | 'text'
  | 'secret'
  | 'url'
  | 'bool'
  | 'number'
  | 'file_path'
  | 'enum'

export interface UiValidation {
  required: boolean
  min_length: number | null
  max_length: number | null
  pattern: string | null
}

export interface UiFieldSchema {
  kind: FieldKindKey
  enum_values: string[] | null
  advanced: boolean
  help_url: string | null
  depends_on: string | null
  validation: UiValidation
}

export interface ServiceEnvVar {
  name: string
  description: string
  example: string
  secret: boolean
  required: boolean
  ui?: UiFieldSchema
}

export interface ServiceSchema {
  name: string
  display_name: string
  description: string
  category: string
  supports_multi_instance: boolean
  default_port: number | null
  env: ServiceEnvVar[]
}

export interface SchemaGetResponse {
  services: Record<string, ServiceSchema>
}

// ─── Drafts ─────────────────────────────────────────────────────────────

export interface DraftEntry {
  key: string
  value: string
}

export interface DraftSetOutcome {
  written: number
  skipped: string[]
  backup_path: string | null
}

export interface CommitOutcome {
  written: number
  skipped: string[]
  backup_path: string | null
  audit_pass_count: number
  audit_total_count: number
  // Present when the gate failed; the caller can render the audit body inline.
  ok?: false
  audit?: unknown
}

// ─── Public API ─────────────────────────────────────────────────────────

export const setupApi = {
  state(signal?: AbortSignal): Promise<SetupSnapshot> {
    return setupAction<SetupSnapshot>('state', {}, signal)
  },

  schemaGet(services?: string[], signal?: AbortSignal): Promise<SchemaGetResponse> {
    return setupAction<SchemaGetResponse>('schema.get', services ? { services } : {}, signal)
  },

  draftGet(signal?: AbortSignal): Promise<{ entries: DraftEntry[] }> {
    return setupAction<{ entries: DraftEntry[] }>('draft.get', {}, signal)
  },

  draftSet(
    entries: DraftEntry[],
    options?: { force?: boolean },
    signal?: AbortSignal,
  ): Promise<DraftSetOutcome> {
    return setupAction<DraftSetOutcome>(
      'draft.set',
      { entries, force: options?.force ?? false },
      signal,
    )
  },

  draftCommit(
    options?: { force?: boolean },
    signal?: AbortSignal,
  ): Promise<CommitOutcome> {
    return setupAction<CommitOutcome>(
      'draft.commit',
      { force: options?.force ?? false, confirm: true },
      signal,
    )
  },

  finalize(signal?: AbortSignal): Promise<CommitOutcome> {
    return setupAction<CommitOutcome>('finalize', { confirm: true }, signal)
  },
}
