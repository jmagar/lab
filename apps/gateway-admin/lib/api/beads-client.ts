import { beadsActionUrl } from './gateway-config.ts'
import { performServiceAction, type ServiceActionError } from './service-action-client.ts'
import {
  IssueSchema,
  IssueListResponseSchema,
  type Issue,
  type IssueSummary,
  type IssueListParams,
} from '../types/beads'

// TODO(lab-5t4b.5): write functions (createIssue, updateIssue, closeIssue,
// reopenIssue, addComment, listLabels) are deferred — v1 is read-only.

export class BeadsError extends Error implements ServiceActionError {
  status: number
  code?: string

  constructor(message: string, status: number, code?: string) {
    super(message)
    this.name = 'BeadsError'
    this.status = status
    this.code = code
  }
}

async function beadsAction<T>(
  action: string,
  params: object = {},
  signal?: AbortSignal,
): Promise<T> {
  return performServiceAction<T, BeadsError>({
    action,
    params,
    signal,
    serviceLabel: 'Beads',
    url: beadsActionUrl(),
    createError: (msg, status, code) => new BeadsError(msg, status, code),
  })
}

/**
 * List issues with optional filters.
 *
 * Validates the response shape via the Zod schema using `safeParse` — any
 * fields the backend doesn't recognise are dropped silently rather than
 * forcing the whole call to fail.
 */
export async function listIssues(
  params: IssueListParams = {},
  signal?: AbortSignal,
): Promise<IssueSummary[]> {
  const raw = await beadsAction<unknown>('issue.list', params, signal)
  const parsed = IssueListResponseSchema.safeParse(raw)
  if (!parsed.success) {
    throw new BeadsError(
      `Beads issue.list response did not match schema: ${parsed.error.message}`,
      502,
      'decode_error',
    )
  }
  return parsed.data.issues
}

/**
 * Fetch one issue by id.
 */
export async function getIssue(id: string, signal?: AbortSignal): Promise<Issue> {
  const raw = await beadsAction<unknown>('issue.get', { id }, signal)
  const parsed = IssueSchema.safeParse(raw)
  if (!parsed.success) {
    throw new BeadsError(
      `Beads issue.get response did not match schema: ${parsed.error.message}`,
      502,
      'decode_error',
    )
  }
  return parsed.data
}
